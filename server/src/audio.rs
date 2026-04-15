use crate::config::{AudioConfig, SharedAudioConfig, SharedServerStatus};
use crate::telemetry::SharedState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SAMPLE_RATE: f32 = 44100.0;

struct EffectState {
    abs_active: f32,
    slip_amount: f32,
    landing_intensity: f32,
    landing_phase: f32,
    impact_intensity: f32,
    impact_phase: f32,
    prev_vel_y: f32,
    prev_g_mag: f32,
    // Cached config values (updated periodically)
    cfg: AudioConfig,
}

impl Default for EffectState {
    fn default() -> Self {
        EffectState {
            abs_active: 0.0,
            slip_amount: 0.0,
            landing_intensity: 0.0,
            landing_phase: 0.0,
            impact_intensity: 0.0,
            impact_phase: 0.0,
            prev_vel_y: 0.0,
            prev_g_mag: 0.0,
            cfg: AudioConfig::default(),
        }
    }
}

impl EffectState {
    fn update_from_telemetry(&mut self, state: &crate::telemetry::TelemetryState) {
        self.abs_active = if state.abs > 0.5 { 1.0 } else { 0.0 };
        self.slip_amount = (state.wheel_slip * 3.0).min(1.0);

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
        if !self.cfg.enabled { return 0.0; }

        let mut out = 0.0f32;
        let c = &self.cfg;

        if self.abs_active > 0.0 {
            let pulse = if (t * c.abs_pulse_rate * std::f32::consts::TAU).sin() > 0.0 { 1.0 } else { 0.0 };
            out += (t * c.abs_freq * std::f32::consts::TAU).sin() * c.abs_volume * pulse;
        }

        if self.slip_amount > 0.05 {
            out += (t * c.slip_freq * std::f32::consts::TAU).sin() * c.slip_volume * self.slip_amount;
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

        (out * c.master_volume).clamp(-1.0, 1.0)
    }
}

pub fn list_devices() {
    let host = cpal::default_host();
    eprintln!("[audio] Available output devices:");
    if let Ok(devices) = host.output_devices() {
        for (i, dev) in devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| "?".into());
            eprintln!("  [{}] {}", i, name);
        }
    }
}

pub fn start_audio(
    state: SharedState,
    audio_config: SharedAudioConfig,
    running: Arc<AtomicBool>,
    server_status: SharedServerStatus,
) {
    let host = cpal::default_host();
    let device_name = audio_config.lock().unwrap().device_name.clone();

    let device = if let Some(ref name) = device_name {
        host.output_devices()
            .ok()
            .and_then(|mut devs| devs.find(|d| {
                d.name().map(|n| n.contains(name.as_str())).unwrap_or(false)
            }))
            .unwrap_or_else(|| {
                eprintln!("[audio] Device '{}' not found, using default", name);
                host.default_output_device().expect("No audio device")
            })
    } else {
        host.default_output_device().expect("No audio device")
    };

    let dev_name = device.name().unwrap_or_else(|_| "?".into());
    eprintln!("[audio] Using device: {}", dev_name);

    if let Ok(mut st) = server_status.lock() {
        st.audio_active = true;
        st.audio_device = dev_name.clone();
    }

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE as u32),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut effects = EffectState::default();
    let mut t: f32 = 0.0;
    let dt = 1.0 / SAMPLE_RATE;
    let mut update_counter = 0u32;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            update_counter += data.len() as u32;
            if update_counter >= 735 {
                update_counter = 0;
                // Update config from admin panel
                if let Ok(cfg) = audio_config.lock() {
                    effects.cfg = cfg.clone();
                }
                // Update telemetry
                if let Ok(s) = state.lock() {
                    if s.connected {
                        effects.update_from_telemetry(&s);
                    } else {
                        effects = EffectState {
                            cfg: effects.cfg.clone(),
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
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to start audio stream");
    eprintln!("[audio] Audio engine running");

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    drop(stream);
}
