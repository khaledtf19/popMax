use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use winreg::enums::*;
use winreg::RegKey;

/// Per-user "Run" key — apps listed here start when the user logs in.
const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "PopMax";

/// Persisted app preferences (`%LOCALAPPDATA%\PopMax\settings.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub launch_at_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        // First run starts with auto-start enabled.
        Self {
            launch_at_startup: true,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    crate::utils::get_load_path().map(|dir| dir.join("settings.json"))
}

pub fn load_settings() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    if !path.exists() {
        return Settings::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_settings(settings: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string(settings) {
        let _ = std::fs::write(&path, content);
    }
}

/// The Run key value for this executable, quoted so paths with spaces work.
fn run_value() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(format!("\"{}\"", exe.display()))
}

pub fn is_enabled() -> bool {
    let Some(value) = run_value() else {
        return false;
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(RUN_KEY_PATH) else {
        return false;
    };
    key.get_value::<String, _>(VALUE_NAME)
        .map(|current| current == value)
        .unwrap_or(false)
}

pub fn enable() -> bool {
    let Some(value) = run_value() else {
        return false;
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.create_subkey(RUN_KEY_PATH) {
        Ok((key, _)) => key.set_value(VALUE_NAME, &value).is_ok(),
        Err(_) => false,
    }
}

pub fn disable() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY_PATH, KEY_READ | KEY_WRITE) else {
        return false;
    };
    match key.delete_value(VALUE_NAME) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    }
}

pub fn apply_startup_setting() {
    let settings = load_settings();
    if settings.launch_at_startup {
        let _ = enable();
    } else {
        let _ = disable();
    }
}

pub fn toggle_startup() -> Option<bool> {
    let mut settings = load_settings();
    let next = !settings.launch_at_startup;
    let ok = if next { enable() } else { disable() };
    if ok {
        settings.launch_at_startup = next;
        save_settings(&settings);
        Some(next)
    } else {
        None
    }
}
