use crossbeam_channel::{Receiver, unbounded};
use std::thread;

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{MOD_ALT, RegisterHotKey, UnregisterHotKey, VK_SPACE},
        WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    ToggleLauncher,
}

pub fn start() -> Receiver<HotkeyEvent> {
    let (tx, rx) = unbounded();

    thread::spawn(move || unsafe {
        RegisterHotKey(HWND::default(), 1, MOD_ALT, VK_SPACE.0 as u32)
            .expect("Failed to register hotkey");

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            if msg.message == WM_HOTKEY {
                let _ = tx.send(HotkeyEvent::ToggleLauncher);
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnregisterHotKey(HWND::default(), 1);
    });

    rx
}
