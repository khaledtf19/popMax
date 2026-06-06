use gpui::*;
use gpui_component::{ActiveTheme, Root, ThemeConfig};

use crate::launcher::{Cancel, Confirm, LauncherState, SelectNext, SelectPrev};

mod launcher;
pub mod list;
pub mod types;
pub mod scanner;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new("down", SelectNext, None),
            KeyBinding::new("up", SelectPrev, None),
            KeyBinding::new("enter", Confirm, None),
            KeyBinding::new("escape", Cancel, None),
        ]);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(500.0), px(500.0)),
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
                is_movable: true,
                is_resizable: true,
                is_minimizable: true,
                display_id: None,
                window_background: WindowBackgroundAppearance::Transparent,
                app_id: None,
                window_min_size: None,
                window_decorations: None,
                icon: None,
                tabbing_identifier: None,
            },
            |window, cx| {
                let view = cx.new(|cx| LauncherState::new(window, cx));

                cx.new(|cx| Root::new(view, window, cx).bg(gpui::hsla(0., 0., 0., 0.)))
            },
        )
        .expect("Failed to open window");
    });
}
