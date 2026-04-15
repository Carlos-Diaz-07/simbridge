use crate::codemasters::CodemastersPacket;
use crate::games::GameAdapter;
use crate::shm::SharedMemory;

// --- Kunos shared memory structs (ACC/AC/AC Evo/AC Rally) ---
// All use acpmf_* naming, #[repr(C, packed(4))] alignment.
// Array wheel order: [FL, FR, RL, RR]

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct SPageFilePhysics {
    pub packet_id: i32,              // 0
    pub gas: f32,                    // 4
    pub brake: f32,                  // 8
    pub fuel: f32,                   // 12
    pub gear: i32,                   // 16: 0=R, 1=N, 2=1st, 3=2nd...
    pub rpms: i32,                   // 20
    pub steer_angle: f32,            // 24
    pub speed_kmh: f32,              // 28
    pub velocity: [f32; 3],          // 32: local velocity [x, y, z]
    pub acc_g: [f32; 3],             // 44: g-forces [x, y, z]
    pub wheel_slip: [f32; 4],        // 56: [FL, FR, RL, RR]
    pub wheel_load: [f32; 4],        // 72
    pub wheels_pressure: [f32; 4],   // 88
    pub wheel_angular_speed: [f32; 4], // 104
    pub tyre_wear: [f32; 4],         // 120
    pub tyre_dirty_level: [f32; 4],  // 136
    pub tyre_core_temperature: [f32; 4], // 152
    pub camber_rad: [f32; 4],        // 168
    pub suspension_travel: [f32; 4], // 184: [FL, FR, RL, RR]
    pub drs: f32,                    // 200
    pub tc: f32,                     // 204
    pub heading: f32,                // 208
    pub pitch: f32,                  // 212
    pub roll: f32,                   // 216
    pub cg_height: f32,              // 220
    pub car_damage: [f32; 5],        // 224
    pub number_of_tyres_out: i32,    // 244
    pub pit_limiter_on: i32,         // 248
    pub abs: f32,                    // 252
    pub kers_charge: f32,            // 256
    pub kers_input: f32,             // 260
    pub auto_shifter_on: i32,        // 264
    pub ride_height: [f32; 2],       // 268
    pub turbo_boost: f32,            // 276
    pub ballast: f32,                // 280
    pub air_density: f32,            // 284
    pub air_temp: f32,               // 288
    pub road_temp: f32,              // 292
    pub local_angular_vel: [f32; 3], // 296
    pub final_ff: f32,               // 308
    pub performance_meter: f32,      // 312
    pub engine_brake: i32,           // 316
    pub ers_recovery_level: i32,     // 320
    pub ers_power_level: i32,        // 324
    pub ers_heat_charging: i32,      // 328
    pub ers_is_charging: i32,        // 332
    pub kers_current_kj: f32,        // 336
    pub drs_available: i32,          // 340
    pub drs_enabled: i32,            // 344
    pub brake_temp: [f32; 4],        // 348: [FL, FR, RL, RR]
    pub clutch: f32,                 // 364
    pub tyre_temp_i: [f32; 4],       // 368
    pub tyre_temp_m: [f32; 4],       // 384
    pub tyre_temp_o: [f32; 4],       // 400
    pub is_ai_controlled: i32,       // 416
    pub tyre_contact_point: [[f32; 3]; 4],   // 420
    pub tyre_contact_normal: [[f32; 3]; 4],  // 468
    pub tyre_contact_heading: [[f32; 3]; 4], // 516
    pub brake_bias: f32,             // 564
    pub local_velocity: [f32; 3],    // 568
    pub p2p_activations: i32,        // 580
    pub p2p_status: i32,             // 584
    pub current_max_rpm: i32,        // 588
    pub mz: [f32; 4],               // 592
    pub fx: [f32; 4],               // 608
    pub fy: [f32; 4],               // 624
    pub slip_ratio: [f32; 4],        // 640
    pub slip_angle: [f32; 4],        // 656
    pub tc_in_action: i32,           // 672
    pub abs_in_action: i32,          // 676
    pub suspension_damage: [f32; 4], // 680
    pub tyre_temp: [f32; 4],         // 696
    pub water_temp: f32,             // 712
    pub brake_pressure: [f32; 4],    // 716
    pub front_brake_compound: i32,   // 732
    pub rear_brake_compound: i32,    // 736
    pub pad_life: [f32; 4],          // 740
    pub disc_life: [f32; 4],         // 756
    pub ignition_on: i32,            // 772
    pub starter_engine_on: i32,      // 776
    pub is_engine_running: i32,      // 780
    pub kerb_vibration: f32,         // 784
    pub slip_vibrations: f32,        // 788
    pub g_vibrations: f32,           // 792
    pub abs_vibrations: f32,         // 796
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct SPageFileGraphic {
    pub packet_id: i32,              // 0
    pub status: i32,                 // 4: 0=OFF, 1=REPLAY, 2=LIVE, 3=PAUSE
    pub session: i32,                // 8: 0=PRACTICE, 1=QUALIFY, 2=RACE, 3=HOTLAP
    pub current_time: [u16; 15],     // 12
    pub last_time: [u16; 15],        // 42
    pub best_time: [u16; 15],        // 72
    pub split: [u16; 15],            // 102
    pub completed_laps: i32,         // 132
    pub position: i32,               // 136
    pub i_current_time: i32,         // 140: ms
    pub i_last_time: i32,            // 144: ms
    pub i_best_time: i32,            // 148: ms
    pub session_time_left: f32,      // 152
    pub distance_traveled: f32,      // 156
    pub is_in_pit: i32,              // 160
    pub current_sector_index: i32,   // 164
    pub last_sector_time: i32,       // 168: ms
    pub number_of_laps: i32,         // 172
    pub tyre_compound: [u16; 33],    // 176
    // We only need fields up to here for the bridge.
    // The rest of the struct extends to ~2048 bytes with ACC-specific fields.
    // Pad to avoid reading past the mapped region.
    pub _pad: [u8; 1580],
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct SPageFileStatic {
    pub sm_version: [u16; 15],       // 0
    pub ac_version: [u16; 15],       // 30
    pub number_of_sessions: i32,     // 60
    pub num_cars: i32,               // 64
    pub car_model: [u16; 33],        // 68
    pub track: [u16; 33],            // 134
    pub player_name: [u16; 33],      // 200
    pub player_surname: [u16; 33],   // 266
    pub player_nick: [u16; 33],      // 332
    pub sector_count: i32,           // 398
    pub max_torque: f32,             // 402
    pub max_power: f32,              // 406
    pub max_rpm: i32,                // 410
    pub max_fuel: f32,               // 414
    pub max_suspension_travel: [f32; 4], // 418
    pub tyre_radius: [f32; 4],       // 434
    pub max_turbo_boost: f32,        // 450
    pub _deprecated1: f32,           // 454
    pub _deprecated2: f32,           // 458
    pub penalties_enabled: i32,      // 462
    pub aid_fuel_rate: f32,          // 466
    pub aid_tire_rate: f32,          // 470
    pub aid_mechanical_damage: f32,  // 474
    pub aid_allow_tyre_blankets: i32,// 478
    pub aid_stability: f32,          // 482
    pub aid_auto_clutch: i32,        // 486
    pub aid_auto_blip: i32,          // 490
    pub has_drs: i32,                // 494
    pub has_ers: i32,                // 498
    pub has_kers: i32,               // 502
    pub kers_max_j: f32,             // 506
    pub engine_brake_settings_count: i32, // 510
    pub ers_power_controller_count: i32,  // 514
    pub track_spline_length: f32,    // track length in meters
    // Pad reduced to account for alignment padding the compiler inserts
    // between u16 arrays and i32 fields. Total must stay under 2048.
    pub _pad: [u8; 1500],
}

// Compile-time size checks — must fit within 2048-byte SHM mapping
const _: () = assert!(std::mem::size_of::<SPageFilePhysics>() <= 2048);
const _: () = assert!(std::mem::size_of::<SPageFileGraphic>() <= 2048);
const _: () = assert!(std::mem::size_of::<SPageFileStatic>() <= 2048);

// --- Adapter ---

pub struct KunosAdapter {
    game_name: &'static str,
    physics: Option<SharedMemory>,
    graphics: Option<SharedMemory>,
    statics: Option<SharedMemory>,
    last_packet_id: i32,
    // Cached static data (read once per session)
    max_rpm: f32,
    max_fuel: f32,
    track_length: f32,
    max_gears: f32,
}

impl KunosAdapter {
    pub fn new(game_name: &'static str) -> Self {
        KunosAdapter {
            game_name,
            physics: None,
            graphics: None,
            statics: None,
            last_packet_id: -1,
            max_rpm: 0.0,
            max_fuel: 0.0,
            track_length: 0.0,
            max_gears: 7.0,
        }
    }
}

impl GameAdapter for KunosAdapter {
    fn name(&self) -> &str {
        self.game_name
    }

    fn connect(&mut self) -> bool {
        if self.is_connected() {
            return true;
        }

        let physics = SharedMemory::open("Local\\acpmf_physics", 2048);
        let graphics = SharedMemory::open("Local\\acpmf_graphics", 2048);
        let statics = SharedMemory::open("Local\\acpmf_static", 2048);

        if physics.is_some() && graphics.is_some() && statics.is_some() {
            // Read static data once
            if let Some(ref shm) = statics {
                let s: SPageFileStatic = unsafe { shm.read() };
                self.max_rpm = s.max_rpm as f32;
                self.max_fuel = s.max_fuel;
                self.track_length = s.track_spline_length;
            }

            self.physics = physics;
            self.graphics = graphics;
            self.statics = statics;
            self.last_packet_id = -1;
            true
        } else {
            false
        }
    }

    fn is_connected(&self) -> bool {
        self.physics.is_some()
    }

    fn read(&mut self) -> Option<CodemastersPacket> {
        let physics_shm = self.physics.as_ref()?;
        let graphics_shm = self.graphics.as_ref()?;

        let p: SPageFilePhysics = unsafe { physics_shm.read() };
        let g: SPageFileGraphic = unsafe { graphics_shm.read() };

        // Skip if no new data
        if p.packet_id == self.last_packet_id {
            return None;
        }
        self.last_packet_id = p.packet_id;

        // Re-read static data periodically (session changes)
        if let Some(ref shm) = self.statics {
            let s: SPageFileStatic = unsafe { shm.read() };
            if s.max_rpm > 0 {
                self.max_rpm = s.max_rpm as f32;
                self.max_fuel = s.max_fuel;
                self.track_length = s.track_spline_length;
            }
        }

        let mut pkt = CodemastersPacket::zeroed();

        // --- Map fields ---

        // Speed: ACC gives km/h, Codemasters wants m/s
        pkt.speed = p.speed_kmh / 3.6;

        // RPM
        pkt.engine_rate = p.rpms as f32;

        // Gear: ACC 0=R,1=N,2=1st → Codemasters 10=R,0=N,1=1st
        pkt.gear = match p.gear {
            0 => 10.0, // reverse
            1 => 0.0,  // neutral
            g => (g - 1) as f32,
        };

        // Inputs
        pkt.throttle = p.gas;
        pkt.brake = p.brake;
        pkt.clutch = p.clutch;
        pkt.steer = p.steer_angle; // already -1.0 to 1.0

        // G-forces: ACC acc_g[0]=lateral, acc_g[2]=longitudinal
        pkt.g_force_lat = p.acc_g[0];
        pkt.g_force_lon = p.acc_g[2];

        // Velocity (local frame)
        pkt.vel_x = p.velocity[0];
        pkt.vel_y = p.velocity[1];
        pkt.vel_z = p.velocity[2];

        // Orientation
        pkt.pitch_x = p.pitch.sin();
        pkt.pitch_y = p.pitch.cos();
        pkt.roll_x = p.roll.sin();
        pkt.roll_y = p.roll.cos();

        // Suspension: Kunos [FL,FR,RL,RR] → Codemasters [RL,RR,FL,FR]
        pkt.susp_pos_fl = p.suspension_travel[0];
        pkt.susp_pos_fr = p.suspension_travel[1];
        pkt.susp_pos_rl = p.suspension_travel[2];
        pkt.susp_pos_rr = p.suspension_travel[3];

        // Wheel angular speed: same reorder
        pkt.wheel_speed_fl = p.wheel_angular_speed[0];
        pkt.wheel_speed_fr = p.wheel_angular_speed[1];
        pkt.wheel_speed_rl = p.wheel_angular_speed[2];
        pkt.wheel_speed_rr = p.wheel_angular_speed[3];

        // Brake temps: Kunos [FL,FR,RL,RR] → Codemasters [RL,RR,FL,FR]
        pkt.brakes_temp_fl = p.brake_temp[0];
        pkt.brakes_temp_fr = p.brake_temp[1];
        pkt.brakes_temp_rl = p.brake_temp[2];
        pkt.brakes_temp_rr = p.brake_temp[3];

        // Tyre pressures: same reorder
        pkt.tyres_pressure_fl = p.wheels_pressure[0];
        pkt.tyres_pressure_fr = p.wheels_pressure[1];
        pkt.tyres_pressure_rl = p.wheels_pressure[2];
        pkt.tyres_pressure_rr = p.wheels_pressure[3];

        // Lap data
        pkt.lap = g.completed_laps as f32;
        pkt.lap_time = g.i_current_time as f32 / 1000.0;
        pkt.last_lap_time = g.i_last_time as f32 / 1000.0;
        pkt.lap_distance = g.distance_traveled;

        // Session data
        pkt.car_position = g.position as f32;
        pkt.sector = g.current_sector_index as f32;
        pkt.total_laps = g.number_of_laps as f32;
        pkt.in_pits = g.is_in_pit as f32;

        // Fuel
        pkt.fuel_in_tank = p.fuel;
        pkt.fuel_capacity = self.max_fuel;

        // Static data
        pkt.max_rpm = self.max_rpm;
        pkt.idle_rpm = self.max_rpm * 0.15; // approximate
        pkt.track_size = self.track_length;
        pkt.max_gears = self.max_gears;

        // TC/ABS
        pkt.traction_control = p.tc;
        pkt.anti_lock_brakes = p.abs;

        Some(pkt)
    }

    fn disconnect(&mut self) {
        self.physics = None;
        self.graphics = None;
        self.statics = None;
        self.last_packet_id = -1;
    }
}
