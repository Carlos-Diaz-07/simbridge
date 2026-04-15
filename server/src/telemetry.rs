use simbridge_shared::CodemastersPacket;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

/// Shared telemetry state, updated by UDP receiver, read by dashboard + audio.
#[derive(Clone, serde::Serialize)]
pub struct TelemetryState {
    pub connected: bool,
    pub speed_kmh: f32,
    pub rpm: f32,
    pub max_rpm: f32,
    pub gear: String,
    pub throttle: f32,
    pub brake: f32,
    pub clutch: f32,
    pub g_lat: f32,
    pub g_lon: f32,
    pub lap: i32,
    pub position: i32,
    pub lap_time: f32,
    pub last_lap_time: f32,
    pub fuel: f32,
    pub fuel_capacity: f32,
    pub tyre_pressure: [f32; 4], // FL, FR, RL, RR
    pub brake_temp: [f32; 4],
    pub tc: f32,
    pub abs: f32,
    pub in_pits: bool,
    pub sector: i32,
    pub total_laps: i32,
    // Raw values for audio engine
    pub wheel_slip: f32,
    pub susp_vel: [f32; 4],
    pub vel_y: f32,
}

impl Default for TelemetryState {
    fn default() -> Self {
        TelemetryState {
            connected: false,
            speed_kmh: 0.0,
            rpm: 0.0,
            max_rpm: 8000.0,
            gear: "N".into(),
            throttle: 0.0,
            brake: 0.0,
            clutch: 0.0,
            g_lat: 0.0,
            g_lon: 0.0,
            lap: 0,
            position: 0,
            lap_time: 0.0,
            last_lap_time: 0.0,
            fuel: 0.0,
            fuel_capacity: 0.0,
            tyre_pressure: [0.0; 4],
            brake_temp: [0.0; 4],
            tc: 0.0,
            abs: 0.0,
            in_pits: false,
            sector: 0,
            total_laps: 0,
            wheel_slip: 0.0,
            susp_vel: [0.0; 4],
            vel_y: 0.0,
        }
    }
}

impl TelemetryState {
    pub fn from_packet(pkt: &CodemastersPacket) -> Self {
        let gear_str = pkt.gear_string().to_string();
        TelemetryState {
            connected: true,
            speed_kmh: pkt.speed_kmh(),
            rpm: pkt.engine_rate,
            max_rpm: if pkt.max_rpm > 0.0 { pkt.max_rpm } else { 8000.0 },
            gear: gear_str,
            throttle: pkt.throttle,
            brake: pkt.brake,
            clutch: pkt.clutch,
            g_lat: pkt.g_force_lat,
            g_lon: pkt.g_force_lon,
            lap: pkt.lap as i32,
            position: pkt.car_position as i32,
            lap_time: pkt.lap_time,
            last_lap_time: pkt.last_lap_time,
            fuel: pkt.fuel_in_tank,
            fuel_capacity: pkt.fuel_capacity,
            tyre_pressure: [
                pkt.tyres_pressure_fl,
                pkt.tyres_pressure_fr,
                pkt.tyres_pressure_rl,
                pkt.tyres_pressure_rr,
            ],
            brake_temp: [
                pkt.brakes_temp_fl,
                pkt.brakes_temp_fr,
                pkt.brakes_temp_rl,
                pkt.brakes_temp_rr,
            ],
            tc: pkt.traction_control,
            abs: pkt.anti_lock_brakes,
            in_pits: pkt.in_pits > 0.5,
            sector: pkt.sector as i32,
            total_laps: pkt.total_laps as i32,
            // Derive wheel slip from wheel speed variance
            wheel_slip: {
                let speeds = [
                    pkt.wheel_speed_fl, pkt.wheel_speed_fr,
                    pkt.wheel_speed_rl, pkt.wheel_speed_rr,
                ];
                let avg = speeds.iter().sum::<f32>() / 4.0;
                if avg.abs() > 0.1 {
                    speeds.iter().map(|s| ((s - avg) / avg).abs()).sum::<f32>() / 4.0
                } else {
                    0.0
                }
            },
            susp_vel: [
                pkt.susp_vel_fl, pkt.susp_vel_fr,
                pkt.susp_vel_rl, pkt.susp_vel_rr,
            ],
            vel_y: pkt.vel_y,
        }
    }
}

pub type SharedState = Arc<Mutex<TelemetryState>>;

/// Start UDP receiver thread. Updates shared state at ~60Hz.
pub fn start_receiver(
    port: u16,
    state: SharedState,
    server_status: crate::config::SharedServerStatus,
) {
    thread::spawn(move || {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))
            .unwrap_or_else(|_| panic!("Failed to bind UDP port {}", port));
        sock.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();

        eprintln!("[server] Listening for telemetry on UDP port {}", port);
        let mut buf = [0u8; 512];
        let mut last_data = std::time::Instant::now();
        let mut total_packets: u64 = 0;
        let mut rate_counter: u32 = 0;
        let mut rate_timer = std::time::Instant::now();
        let start_time = std::time::Instant::now();

        loop {
            match sock.recv_from(&mut buf) {
                Ok((264, _)) => {
                    let pkt = CodemastersPacket::from_bytes(
                        buf[..264].try_into().unwrap()
                    );
                    let new_state = TelemetryState::from_packet(&pkt);
                    if let Ok(mut s) = state.lock() {
                        *s = new_state;
                    }
                    total_packets += 1;
                    rate_counter += 1;
                    last_data = std::time::Instant::now();
                }
                Ok((n, _)) => {
                    eprintln!("[server] Unexpected packet size: {} bytes", n);
                }
                Err(_) => {
                    if last_data.elapsed() > std::time::Duration::from_secs(5) {
                        if let Ok(mut s) = state.lock() {
                            s.connected = false;
                        }
                    }
                }
            }

            // Update server status every second
            let rate_elapsed = rate_timer.elapsed();
            if rate_elapsed >= std::time::Duration::from_secs(1) {
                let pps = rate_counter as f32 / rate_elapsed.as_secs_f32();
                rate_counter = 0;
                rate_timer = std::time::Instant::now();

                let connected = last_data.elapsed() < std::time::Duration::from_secs(5);
                if let Ok(mut st) = server_status.lock() {
                    st.bridge_connected = connected;
                    st.packets_received = total_packets;
                    st.packets_per_sec = pps;
                    st.uptime_secs = start_time.elapsed().as_secs();
                    if connected {
                        st.game_name = "Connected".into();
                    } else {
                        st.game_name = "Waiting for bridge...".into();
                    }
                }
            }
        }
    });
}
