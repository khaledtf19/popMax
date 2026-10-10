use crate::config::AppConfig;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, Root, button::Button};

pub struct ThemeSwitcher {
    themes: Vec<SharedString>,
}

impl ThemeSwitcher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut themes: Vec<SharedString> = gpui_component::ThemeRegistry::global(cx)
            .themes()
            .keys()
            .cloned()
            .collect();
        themes.sort();
        Self { themes }
    }
}

impl Render for ThemeSwitcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let themes = self.themes.clone();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .bg(cx.theme().background)
            .size_full()
            .text_color(cx.theme().foreground)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("back_btn")
                            .label("Back")
                            .on_click(|_event, window, _cx| {
                                window.remove_window();
                            }),
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Theme Switcher"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .overflow_y_scrollbar()
                    .children(themes.into_iter().map(|t| {
                        let theme_name = t.clone();
                        Button::new(t.clone()).label(t.clone()).on_click(
                            move |_event, _window, cx| {
                                if let Some(theme_config) =
                                    gpui_component::ThemeRegistry::global(cx)
                                        .themes()
                                        .get(&theme_name)
                                        .cloned()
                                {
                                    gpui_component::Theme::global_mut(cx)
                                        .apply_config(&theme_config);
                                    cx.refresh_windows();

                                    // save config
                                    let mut config = AppConfig::load();
                                    config.theme = theme_name.to_string();
                                    let _ = config.save();
                                }
                            },
                        )
                    })),
            )
    }
}

pub fn open(cx: &mut App) {
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(600.0), px(400.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Theme Switcher".into()),
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
            window_decorations: Some(WindowDecorations::Server),
            icon: None,
            tabbing_identifier: None,
        },
        |window, cx| {
            let view = cx.new(|cx| ThemeSwitcher::new(cx));
            cx.new(|cx| Root::new(view, window, cx))
        },
    )
    .expect("Failed to open theme switcher window");
}
