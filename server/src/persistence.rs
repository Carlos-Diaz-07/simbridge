use crate::config::{AudioConfig, DashConfig, PersistedConfig};
use std::io::Write;
use std::path::PathBuf;

/// Path to the persisted config file.
/// Honours `$XDG_CONFIG_HOME` and falls back to `$HOME/.config`.
fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("simbridge").join("config.json"))
}

/// Load the persisted config from disk. Returns None if no file exists or
/// the file can't be parsed — callers fall back to defaults.
pub fn load() -> Option<PersistedConfig> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<PersistedConfig>(&text) {
        Ok(cfg) => {
            eprintln!("[config] Loaded {}", path.display());
            Some(cfg)
        }
        Err(e) => {
            eprintln!("[config] Failed to parse {}: {} — using defaults", path.display(), e);
            None
        }
    }
}

/// Atomically write the config to disk. Best-effort: errors are logged but
/// not propagated, since persistence failure shouldn't crash the audio thread.
pub fn save(audio: &AudioConfig, dash: &DashConfig) {
    let Some(path) = config_path() else {
        eprintln!("[config] No HOME/XDG_CONFIG_HOME set, skipping save");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("[config] Cannot create {}: {}", parent.display(), e);
            return;
        }
    }
    let cfg = PersistedConfig { audio: audio.clone(), dash: dash.clone() };
    let json = match serde_json::to_string_pretty(&cfg) {
        Ok(s) => s,
        Err(e) => { eprintln!("[config] Serialize failed: {}", e); return; }
    };
    // Write to a sibling temp file then rename for atomic replacement.
    let tmp = path.with_extension("json.tmp");
    let res = (|| -> std::io::Result<()> {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
        std::fs::rename(&tmp, &path)
    })();
    if let Err(e) = res {
        eprintln!("[config] Save to {} failed: {}", path.display(), e);
        let _ = std::fs::remove_file(&tmp);
    }
}
