mod codemasters;
mod games;
mod shm;

use games::GameAdapter;
use games::kunos::KunosAdapter;
use games::rfactor2::RFactor2Adapter;
use games::beamng::BeamNGAdapter;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_PORT: u16 = 20777;
const DEFAULT_IP: &str = "127.0.0.1";
const POLL_RATE_HZ: u64 = 60;
const RECONNECT_INTERVAL: Duration = Duration::from_secs(2);

fn print_usage() {
    eprintln!("simbridge — shared memory to UDP telemetry bridge");
    eprintln!();
    eprintln!("Usage: simbridge.exe <game> [options]");
    eprintln!();
    eprintln!("Games:");
    eprintln!("  acc      Assetto Corsa Competizione");
    eprintln!("  ac       Assetto Corsa");
    eprintln!("  acevo    Assetto Corsa Evo");
    eprintln!("  acrally  Assetto Corsa Rally");
    eprintln!("  rf2      rFactor 2");
    eprintln!("  beamng   BeamNG.drive");
    eprintln!("  auto     Auto-detect running game");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --port <N>   UDP port (default: {})", DEFAULT_PORT);
    eprintln!("  --ip <ADDR>  Target IP (default: {})", DEFAULT_IP);
}

/// Try to detect which game is running by probing shared memory names.
fn auto_detect() -> Option<Box<dyn GameAdapter>> {
    use crate::shm::SharedMemory;

    // Try Kunos family first (most common)
    if SharedMemory::open("Local\\acpmf_physics", 4).is_some() {
        eprintln!("[simbridge] Auto-detected Kunos game (AC/ACC/Evo/Rally)");
        return Some(Box::new(KunosAdapter::new("Kunos (auto-detected)")));
    }
    // Try rFactor 2
    if SharedMemory::open("Local\\$rFactor2SMMP_Telemetry$", 4).is_some() {
        eprintln!("[simbridge] Auto-detected rFactor 2");
        return Some(Box::new(RFactor2Adapter::new()));
    }
    // Try BeamNG
    if SharedMemory::open("Local\\BeamNGSharedMemory", 4).is_some() {
        eprintln!("[simbridge] Auto-detected BeamNG.drive");
        return Some(Box::new(BeamNGAdapter::new()));
    }
    None
}

fn parse_args() -> Option<(Box<dyn GameAdapter>, String, u16)> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return None;
    }

    let adapter: Box<dyn GameAdapter> = match args[1].as_str() {
        "acc" => Box::new(KunosAdapter::new("Assetto Corsa Competizione")),
        "ac" => Box::new(KunosAdapter::new("Assetto Corsa")),
        "acevo" => Box::new(KunosAdapter::new("Assetto Corsa Evo")),
        "acrally" => Box::new(KunosAdapter::new("Assetto Corsa Rally")),
        "rf2" => Box::new(RFactor2Adapter::new()),
        "beamng" => Box::new(BeamNGAdapter::new()),
        "auto" => {
            // Auto-detect will keep trying in the main loop
            Box::new(KunosAdapter::new("auto-detect"))
        }
        "--help" | "-h" => {
            print_usage();
            return None;
        }
        other => {
            eprintln!("Unknown game: {}", other);
            print_usage();
            return None;
        }
    };

    let mut ip = DEFAULT_IP.to_string();
    let mut port = DEFAULT_PORT;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                port = args.get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_PORT);
            }
            "--ip" => {
                i += 1;
                if let Some(addr) = args.get(i) {
                    ip = addr.clone();
                }
            }
            _ => {}
        }
        i += 1;
    }

    Some((adapter, ip, port))
}

fn main() {
    let (mut adapter, ip, port) = match parse_args() {
        Some(args) => args,
        None => std::process::exit(1),
    };

    let is_auto = adapter.name() == "auto-detect";
    let target = format!("{}:{}", ip, port);
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");

    if is_auto {
        eprintln!("[simbridge] Auto-detect mode — probing for any running game");
    } else {
        eprintln!("[simbridge] Game: {}", adapter.name());
    }
    eprintln!("[simbridge] Sending UDP to {}", target);
    eprintln!("[simbridge] Waiting for game shared memory...");

    let tick = Duration::from_micros(1_000_000 / POLL_RATE_HZ);
    let mut connected = false;
    let mut last_reconnect = Instant::now() - RECONNECT_INTERVAL;
    let mut packets_sent: u64 = 0;
    let mut last_status = Instant::now();

    loop {
        let frame_start = Instant::now();

        if !connected {
            if frame_start.duration_since(last_reconnect) >= RECONNECT_INTERVAL {
                last_reconnect = frame_start;

                if is_auto {
                    // Auto-detect: probe all known SHM patterns
                    if let Some(detected) = auto_detect() {
                        adapter = detected;
                        // Fall through to connect
                    }
                }

                if adapter.connect() {
                    connected = true;
                    eprintln!("[simbridge] Connected to {} shared memory", adapter.name());
                }
            }
        }

        if connected {
            match adapter.read() {
                Some((pkt, ext)) => {
                    // Concatenate Codemasters + bridge extension into one 284-byte
                    // datagram. Server detects the extra bytes and uses them when
                    // present; DR2 (which sends 264 directly) stays untouched.
                    let mut buf = [0u8; 264 + 20];
                    buf[..264].copy_from_slice(pkt.as_bytes());
                    buf[264..].copy_from_slice(ext.as_bytes());
                    let _ = socket.send_to(&buf, &target);
                    packets_sent += 1;
                }
                None => {
                    // No new data this frame — check if SHM is still valid
                    if !adapter.is_connected() {
                        eprintln!("[simbridge] Lost connection, waiting for game...");
                        adapter.disconnect();
                        connected = false;
                    }
                }
            }
        }

        // Status line every 10 seconds
        if frame_start.duration_since(last_status) >= Duration::from_secs(10) {
            if connected {
                eprintln!("[simbridge] Running — {} packets sent", packets_sent);
            }
            last_status = frame_start;
        }

        // Sleep to maintain poll rate
        let elapsed = frame_start.elapsed();
        if elapsed < tick {
            thread::sleep(tick - elapsed);
        }
    }
}
