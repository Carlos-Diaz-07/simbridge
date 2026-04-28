use std::process::Command;

#[derive(Clone, Debug, serde::Serialize)]
pub struct PulseSink {
    pub name: String,
    pub description: String,
}

/// Enumerate PulseAudio / PipeWire output sinks via `pactl list sinks`.
/// Returns an empty Vec if pactl is unavailable or fails.
pub fn list_sinks() -> Vec<PulseSink> {
    let out = match Command::new("pactl").args(["list", "sinks"]).output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };
    let text = match std::str::from_utf8(&out) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut sinks = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_desc: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim_start();
        if line.starts_with("Sink #") {
            if let (Some(n), Some(d)) = (cur_name.take(), cur_desc.take()) {
                sinks.push(PulseSink { name: n, description: d });
            }
            cur_name = None;
            cur_desc = None;
        } else if let Some(rest) = line.strip_prefix("Name:") {
            cur_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Description:") {
            cur_desc = Some(rest.trim().to_string());
        }
    }
    if let (Some(n), Some(d)) = (cur_name, cur_desc) {
        sinks.push(PulseSink { name: n, description: d });
    }
    sinks
}

/// Default sink name reported by `pactl get-default-sink`.
pub fn default_sink_name() -> Option<String> {
    let out = Command::new("pactl").args(["get-default-sink"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = std::str::from_utf8(&out.stdout).ok()?.trim();
    if s.is_empty() { None } else { Some(s.to_string()) }
}
