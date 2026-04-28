use crate::codemasters::{BridgeExtension, CodemastersPacket};
use crate::games::GameAdapter;
use crate::shm::SharedMemory;

// --- rFactor 2 shared memory structs ---
// Uses $rFactor2SMMP_* naming from the rF2SharedMemoryMapPlugin.
// The telemetry map is 241,680 bytes containing up to 128 vehicles.
// We only read the player's vehicle.

// Header at the start of each shared memory map
#[allow(dead_code)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2MappedBufferVersionBlock {
    pub version_update_begin: u32,
    pub version_update_end: u32,
}

// Simplified Vec3 for rF2
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Per-wheel data
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2Wheel {
    pub suspension_deflection: f64,
    pub ride_height: f64,
    pub susp_force: f64,
    pub brake_temp: f64,
    pub brake_pressure: f64,
    pub rotation: f64,
    pub lateral_patch_vel: f64,
    pub longitudinal_patch_vel: f64,
    pub lateral_ground_vel: f64,
    pub longitudinal_ground_vel: f64,
    pub camber: f64,
    pub lateral_force: f64,
    pub longitudinal_force: f64,
    pub tire_load: f64,
    pub grip_fract: f64,
    pub pressure: f64,
    pub temperature: [f64; 3], // inner, middle, outer
    pub wear: f64,
    pub terrain_name: [u8; 16],
    pub surface_type: u8,
    pub flat: u8,
    pub detached: u8,
    pub static_undeflected_radius: f64,
    pub vertical_tire_deflection: f64,
    pub wheel_y_location: f64,
    pub toe: f64,
    pub tire_carcass_temperature: f64,
    pub tire_inner_layer_temperature: [f64; 3],
    pub _pad: [u8; 24],
}

// Per-vehicle telemetry (partial — only fields we need)
// Full struct is ~1888 bytes per vehicle.
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2VehicleTelemetry {
    pub id: i32,
    pub delta_time: f64,
    pub elapsed_time: f64,
    pub lap_number: i32,
    pub lap_start_et: f64,
    pub vehicle_name: [u8; 64],
    pub track_name: [u8; 64],

    // Position and orientation
    pub pos: Rf2Vec3,
    pub local_vel: Rf2Vec3,
    pub local_accel: Rf2Vec3,
    pub ori: [Rf2Vec3; 3],
    pub local_rot: Rf2Vec3,
    pub local_rot_accel: Rf2Vec3,

    pub gear: i32,       // -1=R, 0=N, 1+=forward
    pub engine_rpm: f64,
    pub engine_water_temp: f64,
    pub engine_oil_temp: f64,
    pub clutch_rpm: f64,

    pub unfiltered_throttle: f64,
    pub unfiltered_brake: f64,
    pub unfiltered_steering: f64,
    pub unfiltered_clutch: f64,

    pub filtered_throttle: f64,
    pub filtered_brake: f64,
    pub filtered_steering: f64,
    pub filtered_clutch: f64,

    pub steering_shaft_torque: f64,
    pub front_3rd_deflection: f64,
    pub rear_3rd_deflection: f64,

    pub front_wing_height: f64,
    pub front_ride_height: f64,
    pub rear_ride_height: f64,
    pub drag: f64,
    pub front_downforce: f64,
    pub rear_downforce: f64,

    pub fuel: f64,
    pub engine_max_rpm: f64,
    pub scheduled_stops: u8,
    pub overheating: u8,
    pub detached: u8,
    pub headlights: u8,
    pub dent_severity: [u8; 8],
    pub last_impact_et: f64,
    pub last_impact_magnitude: f64,
    pub last_impact_pos: Rf2Vec3,

    pub engine_torque: f64,
    pub current_sector: i32,
    pub speed_limiter: u8,
    pub max_gears: u8,
    pub front_tire_compound_index: u8,
    pub rear_tire_compound_index: u8,
    pub fuel_capacity: f64,
    pub front_flap_activated: u8,
    pub rear_flap_activated: u8,
    pub rear_flap_legal_status: u8,
    pub ignition_starter: u8,
    pub front_tire_compound_name: [u8; 18],
    pub rear_tire_compound_name: [u8; 18],
    pub speed_limiter_available: u8,
    pub anti_stall_activated: u8,
    pub _unused: [u8; 2],
    pub visual_steering_wheel_range: f32,
    pub rear_brake_bias: f64,
    pub turbo_boost_pressure: f64,
    pub physics_to_graphics_offset: [f32; 3],
    pub physics_to_graphics_rotation: [f32; 3],

    pub wheels: [Rf2Wheel; 4], // [FL, FR, RL, RR]
}

// Scoring per-vehicle (partial)
#[allow(dead_code)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2VehicleScoring {
    pub id: i32,
    pub driver_name: [u8; 32],
    pub vehicle_name: [u8; 64],
    pub total_laps: i16,
    pub sector: i8,
    pub finish_status: i8,
    pub lap_dist: f64,
    pub path_lateral: f64,
    pub track_edge: f64,
    pub best_sector1: f64,
    pub best_sector2: f64,
    pub best_lap_time: f64,
    pub last_sector1: f64,
    pub last_sector2: f64,
    pub last_lap_time: f64,
    pub cur_sector1: f64,
    pub cur_sector2: f64,
    pub num_pitstops: i16,
    pub num_penalties: i16,
    pub is_player: u8,
    pub control: i8,
    pub in_pits: u8,
    pub place: u8,
    pub vehicle_class: [u8; 32],
    pub time_behind_next: f64,
    pub laps_behind_next: i32,
    pub time_behind_leader: f64,
    pub laps_behind_leader: i32,
    pub lap_start_et: f64,
    pub pos: Rf2Vec3,
    pub local_vel: Rf2Vec3,
    pub local_accel: Rf2Vec3,
    pub ori_x: Rf2Vec3,
    pub ori_y: Rf2Vec3,
    pub ori_z: Rf2Vec3,
    pub local_rot: Rf2Vec3,
    pub local_rot_accel: Rf2Vec3,
    pub speed: f64,
    // ... more fields but we don't need them
    pub _pad: [u8; 256],
}

// Scoring header
#[allow(dead_code)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Rf2ScoringHeader {
    pub track_name: [u8; 64],
    pub session: i32,
    pub current_et: f64,
    pub end_et: f64,
    pub max_laps: i32,
    pub lap_dist: f64, // track length
    // ... more fields
    pub _pad: [u8; 256],
}

pub struct RFactor2Adapter {
    telemetry_shm: Option<SharedMemory>,
    scoring_shm: Option<SharedMemory>,
    last_elapsed: f64,
}

impl RFactor2Adapter {
    pub fn new() -> Self {
        RFactor2Adapter {
            telemetry_shm: None,
            scoring_shm: None,
            last_elapsed: 0.0,
        }
    }

    fn find_player_telemetry(&self) -> Option<(Rf2VehicleTelemetry, usize)> {
        let shm = self.telemetry_shm.as_ref()?;
        let vehicle_size = std::mem::size_of::<Rf2VehicleTelemetry>();
        let header_size = 8; // version block

        // Scan for player's vehicle (first with valid data)
        for i in 0..128 {
            let offset = header_size + i * vehicle_size;
            if offset + vehicle_size > 241680 {
                break;
            }
            let veh: Rf2VehicleTelemetry = unsafe { shm.read_at(offset) };
            if veh.engine_rpm > 0.0 {
                return Some((veh, i));
            }
        }
        None
    }
}

impl GameAdapter for RFactor2Adapter {
    fn name(&self) -> &str {
        "rFactor 2"
    }

    fn connect(&mut self) -> bool {
        if self.is_connected() {
            return true;
        }

        let telemetry = SharedMemory::open("Local\\$rFactor2SMMP_Telemetry$", 241680);
        let scoring = SharedMemory::open("Local\\$rFactor2SMMP_Scoring$", 75304);

        if telemetry.is_some() && scoring.is_some() {
            self.telemetry_shm = telemetry;
            self.scoring_shm = scoring;
            self.last_elapsed = 0.0;
            true
        } else {
            false
        }
    }

    fn is_connected(&self) -> bool {
        self.telemetry_shm.is_some()
    }

    fn read(&mut self) -> Option<(CodemastersPacket, BridgeExtension)> {
        let (veh, _idx) = self.find_player_telemetry()?;

        // Skip if no new data
        if veh.elapsed_time == self.last_elapsed {
            return None;
        }
        self.last_elapsed = veh.elapsed_time;

        let mut pkt = CodemastersPacket::zeroed();

        // Speed: local_vel magnitude (m/s)
        pkt.speed = ((veh.local_vel.x * veh.local_vel.x
            + veh.local_vel.y * veh.local_vel.y
            + veh.local_vel.z * veh.local_vel.z)
            .sqrt()) as f32;

        pkt.engine_rate = veh.engine_rpm as f32;

        // Gear: rF2 -1=R,0=N,1+=fwd → Codemasters 10=R,0=N,1+=fwd
        pkt.gear = match veh.gear {
            -1 => 10.0,
            0 => 0.0,
            g => g as f32,
        };

        // Inputs
        pkt.throttle = veh.filtered_throttle as f32;
        pkt.brake = veh.filtered_brake as f32;
        pkt.clutch = veh.filtered_clutch as f32;
        pkt.steer = veh.filtered_steering as f32;

        // G-forces from local acceleration (in g's: divide by 9.81)
        pkt.g_force_lat = (veh.local_accel.x / 9.81) as f32;
        pkt.g_force_lon = (veh.local_accel.z / 9.81) as f32;

        // Velocity
        pkt.vel_x = veh.local_vel.x as f32;
        pkt.vel_y = veh.local_vel.y as f32;
        pkt.vel_z = veh.local_vel.z as f32;

        // Position
        pkt.pos_x = veh.pos.x as f32;
        pkt.pos_y = veh.pos.y as f32;
        pkt.pos_z = veh.pos.z as f32;

        // Wheels: rF2 [FL,FR,RL,RR]
        pkt.susp_pos_fl = veh.wheels[0].suspension_deflection as f32;
        pkt.susp_pos_fr = veh.wheels[1].suspension_deflection as f32;
        pkt.susp_pos_rl = veh.wheels[2].suspension_deflection as f32;
        pkt.susp_pos_rr = veh.wheels[3].suspension_deflection as f32;

        pkt.brakes_temp_fl = veh.wheels[0].brake_temp as f32;
        pkt.brakes_temp_fr = veh.wheels[1].brake_temp as f32;
        pkt.brakes_temp_rl = veh.wheels[2].brake_temp as f32;
        pkt.brakes_temp_rr = veh.wheels[3].brake_temp as f32;

        pkt.tyres_pressure_fl = veh.wheels[0].pressure as f32;
        pkt.tyres_pressure_fr = veh.wheels[1].pressure as f32;
        pkt.tyres_pressure_rl = veh.wheels[2].pressure as f32;
        pkt.tyres_pressure_rr = veh.wheels[3].pressure as f32;

        // Fuel
        pkt.fuel_in_tank = veh.fuel as f32;
        pkt.fuel_capacity = veh.fuel_capacity as f32;

        // RPM limits
        pkt.max_rpm = veh.engine_max_rpm as f32;
        pkt.idle_rpm = (veh.engine_max_rpm * 0.15) as f32;
        pkt.max_gears = veh.max_gears as f32;

        // Lap data
        pkt.lap = veh.lap_number as f32;
        pkt.run_time = veh.elapsed_time as f32;
        pkt.sector = veh.current_sector as f32;

        Some((pkt, BridgeExtension::zeroed()))
    }

    fn disconnect(&mut self) {
        self.telemetry_shm = None;
        self.scoring_shm = None;
        self.last_elapsed = 0.0;
    }
}
