use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};

const EMBEDDED_THEMES: &[&str] = &[
    include_str!("themes/adventure.json"),
    include_str!("themes/alduin.json"),
    include_str!("themes/asciinema.json"),
    include_str!("themes/ayu.json"),
    include_str!("themes/catppuccin.json"),
    include_str!("themes/everforest.json"),
    include_str!("themes/fahrenheit.json"),
    include_str!("themes/flexoki.json"),
    include_str!("themes/gruvbox.json"),
    include_str!("themes/harper.json"),
    include_str!("themes/hybrid.json"),
    include_str!("themes/jellybeans.json"),
    include_str!("themes/kibble.json"),
    include_str!("themes/macos-classic.json"),
    include_str!("themes/matrix.json"),
    include_str!("themes/mellifluous.json"),
    include_str!("themes/molokai.json"),
    include_str!("themes/solarized.json"),
    include_str!("themes/spaceduck.json"),
    include_str!("themes/tokyonight.json"),
    include_str!("themes/twilight.json"),
];

pub fn init(cx: &mut App) {
    let theme_name = SharedString::from("Tokyo Night");

    let registry = ThemeRegistry::global_mut(cx);
    for content in EMBEDDED_THEMES {
        if let Err(err) = registry.load_themes_from_str(content) {
            eprintln!("Error loading embedded theme: {err}");
        }
    }
    if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
        Theme::global_mut(cx).apply_config(&theme_config);
        cx.refresh_windows();
    } else {
        eprintln!("Theme not found: {theme_name}");
    }
}
