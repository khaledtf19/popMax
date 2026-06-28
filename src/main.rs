#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::*;
use gpui_component::Root;
use widestring::u16cstr;
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE},
        System::Threading::CreateMutexW,
        UI::WindowsAndMessaging::{FindWindowW, SW_SHOW, SetForegroundWindow, ShowWindow},
    },
    core::PCWSTR,
};

use crate::launcher::{
    Cancel, Confirm, FocusSearch, LauncherState, SelectNext, SelectPrev, ToggleFavorite,
};

mod bangs;
pub mod components;
mod hotkey;
mod launcher;
mod load_themes;
pub mod scanner;
mod tray;
pub mod types;
pub mod utils;
pub mod windows_icons;

const APP_TITLE: &str = "PopMax";

fn main() {
    let Some(_single_instance) = acquire_single_instance() else {
        show_existing_instance();
        return;
    };

    let hotkey_rx = hotkey::start();
    tray::start();

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
                    title: Some(APP_TITLE.into()),
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
                let view = cx.new(|cx| LauncherState::new(window, cx, hotkey_rx.clone()));

                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("Failed to open window");
    });
}

struct SingleInstance(HANDLE);

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn acquire_single_instance() -> Option<SingleInstance> {
    let mutex = unsafe {
        CreateMutexW(
            None,
            false,
            PCWSTR(u16cstr!("Local\\PopMax.SingleInstance").as_ptr()),
        )
        .expect("Failed to create PopMax single-instance mutex")
    };

    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            let _ = CloseHandle(mutex);
        }
        None
    } else {
        Some(SingleInstance(mutex))
    }
}

fn show_existing_instance() {
    let hwnd = unsafe { FindWindowW(None, PCWSTR(u16cstr!("PopMax").as_ptr())) };
    if let Ok(hwnd) = hwnd {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}
