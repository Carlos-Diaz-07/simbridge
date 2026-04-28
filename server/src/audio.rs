use crate::config::{AudioConfig, SharedAudioConfig, SharedServerStatus};
use crate::telemetry::SharedState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

const SAMPLE_RATE: f32 = 44100.0;
// Test pulse for bass shakers / tactile transducers — sub-bass sine you feel,
// not a tone you hear. 50 Hz sits in the sweet spot for most shakers and
// matches the range of the real effects (25–45 Hz).
const BEEP_DURATION_S: f32 = 0.6;
const BEEP_FREQ_HZ: f32 = 50.0;
const BEEP_AMP: f32 = 0.6;

struct EffectState {
    // All intensities are 0..1 and lowpass-smoothed in update_from_telemetry
    // so the haptic output never steps between 60 Hz telemetry updates.
    abs_intensity: f32,
    slip_intensity: f32,
    road_intensity: f32,
    g_intensity: f32,
    engine_intensity: f32,
    // Engine uses a phase accumulator so its frequency can sweep with RPM
    // (gear shifts, etc.) without sample-level discontinuities / clicks.
    engine_freq: f32,
    engine_phase: f32,
    landing_intensity: f32,
    landing_phase: f32,
    impact_intensity: f32,
    impact_phase: f32,
    prev_vel_y: f32,
    prev_g_mag: f32,
    beep_phase: f32,
    beep_active: bool,
    // Cached config values (updated periodically)
    cfg: AudioConfig,
}

impl Default for EffectState {
    fn default() -> Self {
        EffectState {
            abs_intensity: 0.0,
            slip_intensity: 0.0,
            road_intensity: 0.0,
            g_intensity: 0.0,
            engine_intensity: 0.0,
            engine_freq: 30.0,
            engine_phase: 0.0,
            landing_intensity: 0.0,
            landing_phase: 0.0,
            impact_intensity: 0.0,
            impact_phase: 0.0,
            prev_vel_y: 0.0,
            prev_g_mag: 0.0,
            beep_phase: 0.0,
            beep_active: false,
            cfg: AudioConfig::default(),
        }
    }
}

impl EffectState {
    fn update_from_telemetry(&mut self, state: &crate::telemetry::TelemetryState) {
        // Continuous intensities — prefer the bridge's tactile extension when
        // present (ACC kerb/slip/g/abs vibrations are SimHub's haptic sources).
        // Falls back to derived signals so DR2 native UDP still produces feel.
        let target_abs = if state.has_extension {
            (state.abs_vibration * ACC_VIB_GAIN).clamp(0.0, 1.0)
        } else {
            // Proportional, not binary: anti-lock force scaled into 0..1.
            state.abs.clamp(0.0, 1.0)
        };

        // Speed gate — when the car is stopped or paused in a menu the suspension
        // can still oscillate / wheels can have tiny variance. Without this gate
        // those background signals produce a constant baseline rumble.
        // Ramps in from 5 to 25 km/h so there's no hard step.
        let speed_gate = ((state.speed_kmh - 5.0) / 20.0).clamp(0.0, 1.0);

        // ACC's native vibration fields are normalized 0..1 but in practice
        // peak around 0.10–0.20 even during hard driving — much lower than
        // their nominal range suggests. Empirical observation in a Lambo at
        // ~90 km/h full throttle through a corner: slip_vibration ≈ 0.14,
        // g_vibration ≈ 0.10. Multiply by 5× (matches SimHub-style gain)
        // so the vibration_volume sliders cover a useful dynamic range.
        const ACC_VIB_GAIN: f32 = 5.0;

        let target_slip = if state.has_extension {
            (state.slip_vibration * ACC_VIB_GAIN).clamp(0.0, 1.0)
        } else {
            // wheel_slip is wheel-speed variance, noisy from cornering.
            // Subtract a small noise floor before the curve so background
            // variance at low speed doesn't bleed through.
            let s = (state.wheel_slip - 0.005).max(0.0);
            (s * 6.0).clamp(0.0, 1.0).sqrt() * speed_gate
        };

        let target_road = if state.has_extension {
            // Combine kerb-vibration with derived suspension velocity so a
            // smooth track still produces texture proportional to road bumps.
            // avg_susp_vel from ACC is in m/s (frame-delta of suspension
            // travel) — observed range 0.005–0.05, so /0.05 for full.
            let from_kerb = (state.road_vibration * ACC_VIB_GAIN).clamp(0.0, 1.0);
            let from_susp = (state.avg_susp_vel / 0.05).clamp(0.0, 1.0);
            (from_kerb + from_susp * 0.7).min(1.0) * speed_gate
        } else {
            // DR2 native: empirical observation of live telemetry shows
            // suspension velocity values around ±34 at rest and peaks of
            // ±570 during driving — that's only physically sensible as
            // mm/s, not m/s. SpaceMonkey's scale=1000.0 attribute appears
            // to be their downstream consumer's conversion factor, not the
            // wire-format unit. So: mm/s, ~40 noise floor, ~400 mm/s full.
            let mean: f32 = state.susp_vel.iter().map(|v| v.abs()).sum::<f32>() / 4.0;
            ((mean - 40.0) / 400.0).clamp(0.0, 1.0) * speed_gate
        };

        let target_g = if state.has_extension {
            (state.g_vibration * ACC_VIB_GAIN).clamp(0.0, 1.0)
        } else {
            // Cornering load magnitude → soft continuous rumble. ~1.5g = full.
            // 0.4g threshold filters out gravity bias / idle cars on slopes.
            let g_mag = (state.g_lat * state.g_lat + state.g_lon * state.g_lon).sqrt();
            ((g_mag - 0.4) / 1.1).clamp(0.0, 1.0) * speed_gate
        };

        // Engine: pitch from RPM, amplitude from throttle blended with RPM.
        // Idle still produces a faint low rumble (constant 0.15 floor on
        // amplitude); higher RPM + throttle loads the shaker progressively.
        // Both ACC and DR2 populate rpm + max_rpm + throttle so this works
        // identically across games.
        let max_rpm = state.max_rpm.max(1000.0);
        let rpm_norm = (state.rpm / max_rpm).clamp(0.0, 1.2);
        let throttle = state.throttle.clamp(0.0, 1.0);
        let target_engine = if state.rpm > 200.0 {
            ((0.15 + throttle * 0.85) * rpm_norm.min(1.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // RPM/60 = revs per second = base engine fundamental. Clamp into the
        // 20–80 Hz band where bass shakers actually reproduce force.
        let target_engine_freq = (state.rpm / 60.0).clamp(20.0, 80.0);

        // First-order IIR smoothing on every continuous intensity so the
        // 60 Hz telemetry frames don't produce audible amplitude steps.
        // For α=0.35 at fs=60 Hz the −3 dB cutoff is fc = -fs·ln(1-α)/(2π)
        // ≈ 4.1 Hz — comfortably below human tactile change-detection
        // (~8–10 Hz), so transitions feel smooth.
        let alpha = 0.35;
        self.abs_intensity += (target_abs - self.abs_intensity) * alpha;
        self.slip_intensity += (target_slip - self.slip_intensity) * alpha;
        self.road_intensity += (target_road - self.road_intensity) * alpha;
        self.g_intensity += (target_g - self.g_intensity) * alpha;
        self.engine_intensity += (target_engine - self.engine_intensity) * alpha;
        // Engine freq smoothed slower so RPM swings sweep, not jump.
        self.engine_freq += (target_engine_freq - self.engine_freq) * 0.15;

        // Event-driven landing & impact stay as one-shot envelopes.
        let vel_y = state.vel_y;
        if self.prev_vel_y < -2.0 && vel_y.abs() < 1.0 {
            let impact = (-self.prev_vel_y - 2.0).min(10.0) / 10.0;
            self.landing_intensity = impact.max(self.landing_intensity);
            self.landing_phase = 0.0;
        }
        self.prev_vel_y = vel_y;

        let g_mag = (state.g_lat * state.g_lat + state.g_lon * state.g_lon).sqrt();
        let g_delta = (g_mag - self.prev_g_mag).max(0.0);
        if g_delta > 2.0 {
            let impact = ((g_delta - 2.0) / 5.0).min(1.0);
            self.impact_intensity = impact.max(self.impact_intensity);
            self.impact_phase = 0.0;
        }
        self.prev_g_mag = g_mag;

        self.landing_intensity *= 0.95;
        if self.landing_intensity < 0.01 { self.landing_intensity = 0.0; }
        self.impact_intensity *= 0.93;
        if self.impact_intensity < 0.01 { self.impact_intensity = 0.0; }
    }

    fn generate_sample(&mut self, t: f32) -> f32 {
        // Test beep is independent of cfg.enabled and master_volume so the
        // user can verify routing even with effects muted.
        let beep = if self.beep_active {
            self.beep_phase += 1.0 / SAMPLE_RATE;
            if self.beep_phase >= BEEP_DURATION_S {
                self.beep_active = false;
                0.0
            } else {
                let env = if self.beep_phase < 0.02 {
                    self.beep_phase / 0.02
                } else if self.beep_phase > BEEP_DURATION_S - 0.02 {
                    (BEEP_DURATION_S - self.beep_phase) / 0.02
                } else {
                    1.0
                };
                (self.beep_phase * BEEP_FREQ_HZ * std::f32::consts::TAU).sin() * BEEP_AMP * env
            }
        } else {
            0.0
        };

        if !self.cfg.enabled { return beep.clamp(-1.0, 1.0); }

        let mut out = 0.0f32;
        let c = &self.cfg;
        let tau = std::f32::consts::TAU;

        // Continuous effects — no thresholds, intensity is already smoothed.
        // Final * c.master_volume happens once at the bottom; don't double-scale.
        if self.road_intensity > 0.001 {
            out += (t * c.road_freq * tau).sin() * c.road_volume * self.road_intensity;
        }

        // Cornering load: deep continuous rumble proportional to G.
        if self.g_intensity > 0.001 {
            out += (t * c.g_freq * tau).sin() * c.g_volume * self.g_intensity;
        }

        // Engine: phase-accumulator sine so frequency can sweep with RPM
        // (gear shifts, blips) without phase discontinuities.
        if self.engine_intensity > 0.001 {
            self.engine_phase += self.engine_freq / SAMPLE_RATE;
            if self.engine_phase >= 1.0 { self.engine_phase -= 1.0; }
            out += (self.engine_phase * tau).sin() * c.engine_volume * self.engine_intensity;
        }

        // ABS pulse: half-wave rectified pulse at abs_pulse_rate, scaled by intensity.
        if self.abs_intensity > 0.001 {
            let pulse = if (t * c.abs_pulse_rate * tau).sin() > 0.0 { 1.0 } else { 0.0 };
            out += (t * c.abs_freq * tau).sin() * c.abs_volume * pulse * self.abs_intensity;
        }

        // Slip: continuous tone, scales smoothly from zero — no on/off gate.
        if self.slip_intensity > 0.001 {
            out += (t * c.slip_freq * tau).sin() * c.slip_volume * self.slip_intensity;
        }

        if self.landing_intensity > 0.0 {
            self.landing_phase += 1.0 / SAMPLE_RATE;
            let env = (-self.landing_phase * 15.0).exp();
            out += (self.landing_phase * c.landing_freq * std::f32::consts::TAU).sin()
                * self.landing_intensity * env * c.landing_volume;
        }

        if self.impact_intensity > 0.0 {
            self.impact_phase += 1.0 / SAMPLE_RATE;
            let env = (-self.impact_phase * 8.0).exp();
            let sine = (self.impact_phase * c.impact_freq * std::f32::consts::TAU).sin();
            let noise = ((t * 100000.0) as u32).wrapping_mul(1103515245).wrapping_add(12345);
            let noise_f = (noise as f32 / u32::MAX as f32) * 2.0 - 1.0;
            out += (sine * 0.6 + noise_f * 0.4) * self.impact_intensity * env * c.impact_volume;
        }

        (out * c.master_volume + beep).clamp(-1.0, 1.0)
    }
}

pub fn list_devices() {
    let host = cpal::default_host();
    let sinks = crate::pulse::list_sinks();
    if !sinks.is_empty() {
        eprintln!("[audio] PipeWire/PulseAudio sinks (recommended):");
        for (i, s) in sinks.iter().enumerate() {
            eprintln!("  [{}] {}  ({})", i, s.description, s.name);
        }
        eprintln!();
    }
    eprintln!("[audio] Raw ALSA outputs (fallback):");
    if let Ok(devices) = host.output_devices() {
        for (i, dev) in devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| "?".into());
            eprintln!("  [{}] {}", i, name);
        }
    }
}

/// Resolve the desired device_name string into (cpal device, friendly display name).
/// Side effect: sets PULSE_SINK env var when a pulse sink is targeted, so the cpal
/// "pulse" virtual device routes to the right sink.
fn resolve_device(
    host: &cpal::Host,
    device_name: Option<&str>,
) -> Option<(cpal::Device, String)> {
    let pulse_sinks = crate::pulse::list_sinks();
    let pulse_target = device_name.and_then(|name| {
        pulse_sinks.iter().find(|s| s.name == name).cloned()
    });

    if let Some(sink) = pulse_target {
        std::env::set_var("PULSE_SINK", &sink.name);
        let dev = host.output_devices().ok()
            .and_then(|mut devs| devs.find(|d| d.name().map(|n| n == "pulse").unwrap_or(false)))
            .or_else(|| host.output_devices().ok().and_then(|mut devs| {
                devs.find(|d| d.name().map(|n| n == "pipewire").unwrap_or(false))
            }))
            .or_else(|| host.default_output_device())?;
        Some((dev, sink.description))
    } else if let Some(name) = device_name {
        // PULSE_SINK from a previous selection would otherwise force the
        // pulse plugin to use the old sink even when picking a raw ALSA device.
        std::env::remove_var("PULSE_SINK");
        let dev = host.output_devices().ok()
            .and_then(|mut devs| devs.find(|d| {
                d.name().map(|n| n.contains(name)).unwrap_or(false)
            }))
            .or_else(|| {
                eprintln!("[audio] Device '{}' not found, using default", name);
                host.default_output_device()
            })?;
        let dn = dev.name().unwrap_or_else(|_| name.to_string());
        Some((dev, dn))
    } else {
        std::env::remove_var("PULSE_SINK");
        let dev = host.default_output_device()?;
        let dn = dev.name().unwrap_or_else(|_| "default".into());
        Some((dev, dn))
    }
}

/// Build and start a cpal output stream targeting the resolved device.
/// Returns the live stream — drop it to stop playback.
fn build_stream(
    device: &cpal::Device,
    state: SharedState,
    audio_config: SharedAudioConfig,
    server_status: SharedServerStatus,
    test_beep: Arc<AtomicU32>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE as u32),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut effects = EffectState::default();
    let mut t: f32 = 0.0;
    let dt = 1.0 / SAMPLE_RATE;
    let mut update_counter = 0u32;
    let mut last_beep_count = test_beep.load(Ordering::Relaxed);
    let _ = server_status; // status is updated in start_audio outer loop

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            update_counter += data.len() as u32;
            if update_counter >= 735 {
                update_counter = 0;
                let cur_beep = test_beep.load(Ordering::Relaxed);
                if cur_beep != last_beep_count {
                    last_beep_count = cur_beep;
                    effects.beep_phase = 0.0;
                    effects.beep_active = true;
                }
                if let Ok(cfg) = audio_config.lock() {
                    effects.cfg = cfg.clone();
                }
                if let Ok(s) = state.lock() {
                    if s.connected {
                        effects.update_from_telemetry(&s);
                    } else {
                        effects = EffectState {
                            cfg: effects.cfg.clone(),
                            beep_active: effects.beep_active,
                            beep_phase: effects.beep_phase,
                            ..EffectState::default()
                        };
                    }
                }
            }

            for sample in data.iter_mut() {
                *sample = effects.generate_sample(t);
                t += dt;
                if t > 1000.0 { t -= 1000.0; }
            }
        },
        |err| eprintln!("[audio] Stream error: {}", err),
        None,
    )?;
    stream.play().map_err(|_| cpal::BuildStreamError::DeviceNotAvailable)?;
    Ok(stream)
}

pub fn start_audio(
    state: SharedState,
    audio_config: SharedAudioConfig,
    running: Arc<AtomicBool>,
    server_status: SharedServerStatus,
    test_beep: Arc<AtomicU32>,
) {
    let host = cpal::default_host();
    let mut active_name: Option<String> = None;
    let mut stream: Option<cpal::Stream> = None;

    while running.load(Ordering::Relaxed) {
        let desired = audio_config.lock().unwrap().device_name.clone();
        if stream.is_none() || desired != active_name {
            // Drop the old stream first so the audio backend releases the device
            // before we open the new one.
            if stream.is_some() {
                eprintln!("[audio] Device change requested, switching output");
            }
            drop(stream.take());
            if let Ok(mut st) = server_status.lock() {
                st.audio_active = false;
            }
            match resolve_device(&host, desired.as_deref()) {
                Some((dev, display_name)) => {
                    match build_stream(
                        &dev,
                        state.clone(),
                        audio_config.clone(),
                        server_status.clone(),
                        test_beep.clone(),
                    ) {
                        Ok(s) => {
                            eprintln!("[audio] Using device: {}", display_name);
                            if let Ok(mut st) = server_status.lock() {
                                st.audio_active = true;
                                st.audio_device = display_name;
                            }
                            stream = Some(s);
                            active_name = desired;
                        }
                        Err(e) => {
                            eprintln!("[audio] Failed to build stream on '{}': {} — retrying in 2s", display_name, e);
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                    }
                }
                None => {
                    eprintln!("[audio] No audio device available, retrying in 2s");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    drop(stream);
}
