use crate::utils::get_load_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "Tokyo Night".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let Some(mut path) = get_load_path() else {
            return Self::default();
        };
        path.push("config.json");

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let Some(mut path) = get_load_path() else {
            return;
        };
        let _ = fs::create_dir_all(&path);
        path.push("config.json");

        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, content);
        }
    }
}
