use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;

use crate::utils::asset_path;

/// Returns true if the directory exists and contains at least one .json file.
fn dir_has_json_files(dir: &PathBuf) -> bool {
    if !dir.is_dir() {
        return false;
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("json")
            })
        })
        .unwrap_or(false)
}

pub fn init(cx: &mut App) {
    let theme_name = SharedString::from("Tokyo Night");
    let themes_dir = asset_path("themes")
        .filter(|p| dir_has_json_files(p))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/themes"));

    if let Err(err) = ThemeRegistry::watch_dir(themes_dir, cx, move |cx| {
        if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme_config);
            cx.refresh_windows();
        } else {
            eprintln!("Theme not found: {theme_name}");
        }
    }) {
        eprintln!("Error loading themes: {err}");
    }
}
