use std::sync::{Arc, Mutex};

/// Runtime-adjustable settings, shared between admin UI and audio engine.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub master_volume: f32,   // 0.0-1.0
    pub abs_volume: f32,
    pub abs_freq: f32,        // Hz
    pub abs_pulse_rate: f32,  // Hz
    pub slip_volume: f32,
    pub slip_freq: f32,
    pub landing_volume: f32,
    pub landing_freq: f32,
    pub impact_volume: f32,
    pub impact_freq: f32,
    pub device_name: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            enabled: true,
            master_volume: 0.7,
            abs_volume: 0.6,
            abs_freq: 30.0,
            abs_pulse_rate: 10.0,
            slip_volume: 0.5,
            slip_freq: 25.0,
            landing_volume: 0.8,
            landing_freq: 45.0,
            impact_volume: 0.7,
            impact_freq: 35.0,
            device_name: None,
        }
    }
}

/// Server status info for the admin panel.
#[derive(Clone, serde::Serialize)]
pub struct ServerStatus {
    pub bridge_connected: bool,
    pub game_name: String,
    pub packets_received: u64,
    pub packets_per_sec: f32,
    pub dashboard_clients: u32,
    pub audio_active: bool,
    pub audio_device: String,
    pub uptime_secs: u64,
}

impl Default for ServerStatus {
    fn default() -> Self {
        ServerStatus {
            bridge_connected: false,
            game_name: "None".into(),
            packets_received: 0,
            packets_per_sec: 0.0,
            dashboard_clients: 0,
            audio_active: false,
            audio_device: "None".into(),
            uptime_secs: 0,
        }
    }
}

/// Dashboard mode selection — shared between admin panel and dash page.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DashConfig {
    pub mode: String, // "circuit" or "rally"
}

impl Default for DashConfig {
    fn default() -> Self {
        DashConfig { mode: "circuit".into() }
    }
}

pub type SharedAudioConfig = Arc<Mutex<AudioConfig>>;
pub type SharedServerStatus = Arc<Mutex<ServerStatus>>;
pub type SharedDashConfig = Arc<Mutex<DashConfig>>;
