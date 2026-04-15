#!/usr/bin/env python3
"""Listen on UDP 20777 and print decoded Codemasters telemetry packets."""
import socket
import struct
import time

FIELDS = [
    "run_time", "lap_time", "lap_distance", "total_distance",
    "pos_x", "pos_y", "pos_z", "speed_ms",
    "vel_x", "vel_y", "vel_z",
    "roll_x", "roll_y", "roll_z",
    "pitch_x", "pitch_y", "pitch_z",
    "susp_pos_rl", "susp_pos_rr", "susp_pos_fl", "susp_pos_fr",
    "susp_vel_rl", "susp_vel_rr", "susp_vel_fl", "susp_vel_fr",
    "wheel_spd_rl", "wheel_spd_rr", "wheel_spd_fl", "wheel_spd_fr",
    "throttle", "steer", "brake", "clutch",
    "gear", "g_lat", "g_lon", "lap", "rpm",
    "sli_pro", "car_pos", "kers", "kers_max",
    "drs", "tc", "abs", "fuel", "fuel_cap",
    "in_pits", "sector", "sector1_t", "sector2_t",
    "brake_t_rl", "brake_t_rr", "brake_t_fl", "brake_t_fr",
    "tyre_p_rl", "tyre_p_rr", "tyre_p_fl", "tyre_p_fr",
    "team", "total_laps", "track_len", "last_lap_t",
    "max_rpm", "idle_rpm", "max_gears",
]

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(("0.0.0.0", 20777))
sock.settimeout(1.0)

print(f"Listening on UDP port 20777... (expecting 264-byte packets)")
print(f"Launch ACC with simbridge, then drive onto track.\n")

count = 0
last_print = 0

while True:
    try:
        data, addr = sock.recvfrom(512)
    except socket.timeout:
        continue

    count += 1
    now = time.time()

    if len(data) != 264:
        print(f"[{count}] Unexpected packet size: {len(data)} bytes from {addr}")
        continue

    values = struct.unpack("<66f", data)

    # Print summary every 0.5s
    if now - last_print >= 0.5:
        last_print = now
        speed_kmh = values[7] * 3.6
        rpm = values[37]
        gear = int(values[33])
        gear_str = "R" if gear == 10 else ("N" if gear == 0 else str(gear))
        throttle = values[29] * 100
        brake = values[31] * 100
        fuel = values[44]
        g_lat = values[34]
        g_lon = values[35]
        lap = int(values[36])
        pos = int(values[39])

        print(
            f"[pkts:{count:>6}] "
            f"SPD:{speed_kmh:6.1f}km/h  "
            f"RPM:{rpm:6.0f}  "
            f"GEAR:{gear_str:>2}  "
            f"THR:{throttle:5.1f}%  "
            f"BRK:{brake:5.1f}%  "
            f"FUEL:{fuel:5.1f}L  "
            f"G:{g_lat:+5.2f}/{g_lon:+5.2f}  "
            f"LAP:{lap} P{pos}"
        )
