use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;

pub fn init(cx: &mut App) {
    let theme_name = SharedString::from("Tokyo Night");
    let themes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/themes");

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
