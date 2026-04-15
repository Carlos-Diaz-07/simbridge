use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use axum::Json;
use crate::config::{AudioConfig, DashConfig, SharedAudioConfig, SharedDashConfig, SharedServerStatus};
use crate::telemetry::SharedState;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub telemetry: SharedState,
    pub audio_config: SharedAudioConfig,
    pub server_status: SharedServerStatus,
    pub dash_config: SharedDashConfig,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(admin_page))
        .route("/dash", get(dash_page))
        .route("/ws", get(ws_telemetry))
        .route("/ws/admin", get(ws_admin))
        .route("/api/audio", get(get_audio_config).post(set_audio_config))
        .route("/api/audio/devices", get(list_audio_devices))
        .route("/api/dash-url", get(get_dash_url))
        .route("/api/dash-mode", get(get_dash_mode).post(set_dash_mode))
        .route("/dash/lite", get(lite_page))
        .route("/api/telemetry", get(get_telemetry))
        .with_state(state)
}

async fn admin_page() -> Html<&'static str> {
    Html(include_str!("../static/admin.html"))
}

async fn dash_page(State(state): State<AppState>) -> Html<&'static str> {
    let mode = state.dash_config.lock().unwrap().mode.clone();
    match mode.as_str() {
        "rally" => Html(include_str!("../static/rally.html")),
        _ => Html(include_str!("../static/index.html")),
    }
}

// Telemetry WebSocket (for racing dashboard)
async fn ws_telemetry(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| ws_telemetry_conn(socket, state.telemetry))
}

async fn ws_telemetry_conn(mut socket: WebSocket, state: SharedState) {
    loop {
        let json = {
            let s = state.lock().unwrap();
            serde_json::to_string(&*s).unwrap()
        };
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}

// Admin WebSocket (for control panel status)
async fn ws_admin(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| ws_admin_conn(socket, state))
}

async fn ws_admin_conn(mut socket: WebSocket, state: AppState) {
    loop {
        let json = {
            let status = state.server_status.lock().unwrap();
            let telem = state.telemetry.lock().unwrap();
            serde_json::json!({
                "status": *status,
                "telemetry_snapshot": {
                    "connected": telem.connected,
                    "speed_kmh": telem.speed_kmh,
                    "rpm": telem.rpm,
                    "gear": telem.gear,
                }
            }).to_string()
        };
        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await; // 2Hz for admin
    }
}

async fn get_audio_config(State(state): State<AppState>) -> Json<AudioConfig> {
    let cfg = state.audio_config.lock().unwrap().clone();
    Json(cfg)
}

async fn set_audio_config(
    State(state): State<AppState>,
    Json(new_cfg): Json<AudioConfig>,
) -> Json<AudioConfig> {
    let mut cfg = state.audio_config.lock().unwrap();
    *cfg = new_cfg.clone();
    Json(new_cfg)
}

async fn list_audio_devices() -> Json<Vec<String>> {
    use cpal::traits::{HostTrait, DeviceTrait};
    let host = cpal::default_host();
    let devices: Vec<String> = host.output_devices()
        .map(|devs| devs.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default();
    Json(devices)
}

async fn get_dash_url(req: axum::extract::Request) -> Json<serde_json::Value> {
    let port = req.uri().authority().map(|a| a.port_u16().unwrap_or(8888)).unwrap_or(8888);
    // Get LAN IPs
    let ips = get_local_ips();
    let urls: Vec<String> = ips.iter().map(|ip| format!("http://{}:{}/dash", ip, port)).collect();
    Json(serde_json::json!({ "urls": urls, "port": port }))
}

fn get_local_ips() -> Vec<String> {
    use std::net::UdpSocket;
    let mut ips = Vec::new();
    // Connect to a public address to find the default outgoing interface IP
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                ips.push(addr.ip().to_string());
            }
        }
    }
    if ips.is_empty() {
        ips.push("127.0.0.1".into());
    }
    ips
}

async fn get_dash_mode(State(state): State<AppState>) -> Json<DashConfig> {
    let cfg = state.dash_config.lock().unwrap().clone();
    Json(cfg)
}

async fn set_dash_mode(
    State(state): State<AppState>,
    Json(new_cfg): Json<DashConfig>,
) -> Json<DashConfig> {
    let mut cfg = state.dash_config.lock().unwrap();
    *cfg = new_cfg.clone();
    Json(new_cfg)
}

async fn lite_page() -> Html<&'static str> {
    Html(include_str!("../static/lite.html"))
}

async fn get_telemetry(State(state): State<AppState>) -> Json<crate::telemetry::TelemetryState> {
    let s = state.telemetry.lock().unwrap().clone();
    Json(s)
}
