use gpui::*;
use gpui_component::Root;

use crate::launcher::{
    Cancel, Confirm, FocusSearch, LauncherState, SelectNext, SelectPrev, ToggleFavorite,
};

pub mod components;
mod launcher;
mod load_themes;
pub mod scanner;
pub mod types;
pub mod utils;
pub mod windows_icons;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);
        load_themes::init(cx);

        cx.bind_keys([
            KeyBinding::new("down", SelectNext, None),
            KeyBinding::new("up", SelectPrev, None),
            KeyBinding::new("enter", Confirm, None),
            KeyBinding::new("escape", Cancel, None),
            KeyBinding::new("ctrl-d", ToggleFavorite, None),
            KeyBinding::new("ctrl-k", FocusSearch, None),
        ]);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(600.0), px(500.0)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: false,
                is_resizable: false,
                is_minimizable: false,
                display_id: None,
                window_background: WindowBackgroundAppearance::Transparent,
                app_id: None,
                window_min_size: None,
                window_decorations: Some(WindowDecorations::Client),
                icon: None,
                tabbing_identifier: None,
            },
            |window, cx| {
                let view = cx.new(|cx| LauncherState::new(window, cx));

                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("Failed to open window");
    });
}
