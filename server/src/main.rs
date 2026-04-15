mod audio;
mod config;
mod dashboard;
mod telemetry;

use config::{AudioConfig, DashConfig, ServerStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use telemetry::TelemetryState;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut udp_port: u16 = 20777;
    let mut http_port: u16 = 8888;
    let mut audio_device: Option<String> = None;
    let mut no_audio = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--udp-port" => { i += 1; udp_port = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(20777); }
            "--http-port" => { i += 1; http_port = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(8888); }
            "--audio-device" => { i += 1; audio_device = args.get(i).cloned(); }
            "--no-audio" => { no_audio = true; }
            "--list-audio" => { audio::list_devices(); return; }
            "--help" | "-h" => {
                eprintln!("simbridge-server — telemetry dashboard + haptic audio");
                eprintln!();
                eprintln!("Routes:");
                eprintln!("  /          Admin control panel");
                eprintln!("  /dash      Racing dashboard (open on phone)");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  --udp-port <N>         Telemetry UDP port (default: 20777)");
                eprintln!("  --http-port <N>        Dashboard HTTP port (default: 8888)");
                eprintln!("  --audio-device <NAME>  Audio device name (substring match)");
                eprintln!("  --no-audio             Disable audio output");
                eprintln!("  --list-audio           List available audio devices");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let telemetry_state = Arc::new(Mutex::new(TelemetryState::default()));
    let audio_config = Arc::new(Mutex::new(AudioConfig {
        device_name: audio_device.clone(),
        enabled: !no_audio,
        ..AudioConfig::default()
    }));
    let server_status = Arc::new(Mutex::new(ServerStatus::default()));
    let running = Arc::new(AtomicBool::new(true));

    // Start UDP receiver
    telemetry::start_receiver(udp_port, telemetry_state.clone(), server_status.clone());

    // Start audio engine
    if !no_audio {
        let audio_state = telemetry_state.clone();
        let audio_cfg = audio_config.clone();
        let audio_running = running.clone();
        let audio_status = server_status.clone();
        std::thread::spawn(move || {
            audio::start_audio(audio_state, audio_cfg, audio_running, audio_status);
        });
    }

    // Start web server
    let dash_config = Arc::new(Mutex::new(DashConfig::default()));
    let app_state = dashboard::AppState {
        telemetry: telemetry_state,
        audio_config,
        server_status,
        dash_config,
    };
    let app = dashboard::router(app_state);
    let addr = format!("0.0.0.0:{}", http_port);

    eprintln!("[server] Admin panel:  http://localhost:{}", http_port);
    eprintln!("[server] Dashboard:    http://<phone-ip>:{}/dash", http_port);
    eprintln!("[server] UDP port:     {}", udp_port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    running.store(false, Ordering::Relaxed);
}
