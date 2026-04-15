/// Codemasters/Dirt Rally 2.0 UDP telemetry packet.
/// 264 bytes (66 x f32). This is the wire format between bridge and server.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct CodemastersPacket {
    pub run_time: f32,
    pub lap_time: f32,
    pub lap_distance: f32,
    pub total_distance: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub speed: f32,              // m/s
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub roll_x: f32,
    pub roll_y: f32,
    pub roll_z: f32,
    pub pitch_x: f32,
    pub pitch_y: f32,
    pub pitch_z: f32,
    pub susp_pos_rl: f32,
    pub susp_pos_rr: f32,
    pub susp_pos_fl: f32,
    pub susp_pos_fr: f32,
    pub susp_vel_rl: f32,
    pub susp_vel_rr: f32,
    pub susp_vel_fl: f32,
    pub susp_vel_fr: f32,
    pub wheel_speed_rl: f32,
    pub wheel_speed_rr: f32,
    pub wheel_speed_fl: f32,
    pub wheel_speed_fr: f32,
    pub throttle: f32,
    pub steer: f32,
    pub brake: f32,
    pub clutch: f32,
    pub gear: f32,               // 0=N, 1-7=fwd, 10=R
    pub g_force_lat: f32,
    pub g_force_lon: f32,
    pub lap: f32,
    pub engine_rate: f32,        // RPM
    pub sli_pro_native: f32,
    pub car_position: f32,
    pub kers_level: f32,
    pub kers_max_level: f32,
    pub drs: f32,
    pub traction_control: f32,
    pub anti_lock_brakes: f32,
    pub fuel_in_tank: f32,
    pub fuel_capacity: f32,
    pub in_pits: f32,
    pub sector: f32,
    pub sector1_time: f32,
    pub sector2_time: f32,
    pub brakes_temp_rl: f32,
    pub brakes_temp_rr: f32,
    pub brakes_temp_fl: f32,
    pub brakes_temp_fr: f32,
    pub tyres_pressure_rl: f32,
    pub tyres_pressure_rr: f32,
    pub tyres_pressure_fl: f32,
    pub tyres_pressure_fr: f32,
    pub team_info: f32,
    pub total_laps: f32,
    pub track_size: f32,
    pub last_lap_time: f32,
    pub max_rpm: f32,
    pub idle_rpm: f32,
    pub max_gears: f32,
}

const _: () = assert!(std::mem::size_of::<CodemastersPacket>() == 264);

impl CodemastersPacket {
    pub fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                264,
            )
        }
    }

    pub fn from_bytes(bytes: &[u8; 264]) -> Self {
        unsafe { std::ptr::read(bytes.as_ptr() as *const Self) }
    }

    pub fn speed_kmh(&self) -> f32 {
        self.speed * 3.6
    }

    pub fn gear_string(&self) -> &'static str {
        match self.gear as i32 {
            -1 | 10 => "R",
            0 => "N",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            _ => "?",
        }
    }
}
