use crate::codemasters::CodemastersPacket;
use crate::games::GameAdapter;
use crate::shm::SharedMemory;

// --- BeamNG.drive shared memory ---
// BeamNG uses a custom shared memory interface named "Local\\BeamNGSharedMemory".
// The struct layout is based on the OutGauge protocol with extensions.
//
// NOTE: This adapter's struct layout needs verification against a running game.
// BeamNG's SHM format is less documented than Kunos or rF2.
// Fields marked with ? may need offset adjustment.

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct BeamNGData {
    pub magic: u32,              // 0: magic number for validation
    pub speed: f32,              // 4: m/s
    pub rpm: f32,                // 8
    pub max_rpm: f32,            // 12
    pub turbo: f32,              // 16
    pub engine_temp: f32,        // 20
    pub fuel: f32,               // 24: 0.0-1.0
    pub oil_pressure: f32,       // 28
    pub oil_temp: f32,           // 32
    pub dash_lights: u32,        // 36
    pub show_lights: u32,        // 40
    pub throttle: f32,           // 44
    pub brake: f32,              // 48
    pub clutch: f32,             // 52
    pub display1: [u8; 16],      // 56
    pub display2: [u8; 16],      // 72
    pub gear: i32,               // 88: 0=R, 1=N, 2+=forward
    pub _pad: [u8; 128],         // rest of struct
}

pub struct BeamNGAdapter {
    shm: Option<SharedMemory>,
    last_rpm: f32,
}

impl BeamNGAdapter {
    pub fn new() -> Self {
        BeamNGAdapter {
            shm: None,
            last_rpm: 0.0,
        }
    }
}

impl GameAdapter for BeamNGAdapter {
    fn name(&self) -> &str {
        "BeamNG.drive"
    }

    fn connect(&mut self) -> bool {
        if self.is_connected() {
            return true;
        }

        // BeamNG shared memory name — needs verification
        if let Some(shm) = SharedMemory::open("Local\\BeamNGSharedMemory", 4096) {
            self.shm = Some(shm);
            self.last_rpm = 0.0;
            true
        } else {
            false
        }
    }

    fn is_connected(&self) -> bool {
        self.shm.is_some()
    }

    fn read(&mut self) -> Option<CodemastersPacket> {
        let shm = self.shm.as_ref()?;
        let data: BeamNGData = unsafe { shm.read() };

        // Basic change detection
        if data.rpm == self.last_rpm && data.speed == 0.0 {
            return None;
        }
        self.last_rpm = data.rpm;

        let mut pkt = CodemastersPacket::zeroed();

        pkt.speed = data.speed;
        pkt.engine_rate = data.rpm;
        pkt.max_rpm = data.max_rpm;
        pkt.idle_rpm = data.max_rpm * 0.15;
        pkt.throttle = data.throttle;
        pkt.brake = data.brake;
        pkt.clutch = data.clutch;
        pkt.fuel_in_tank = data.fuel; // 0-1 normalized

        // Gear: BeamNG 0=R,1=N,2+=fwd → Codemasters 10=R,0=N,1+=fwd
        pkt.gear = match data.gear {
            0 => 10.0,
            1 => 0.0,
            g => (g - 1) as f32,
        };

        Some(pkt)
    }

    fn disconnect(&mut self) {
        self.shm = None;
        self.last_rpm = 0.0;
    }
}
