use crossbeam_channel::{Receiver, unbounded};
use std::thread;

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{MOD_ALT, RegisterHotKey, UnregisterHotKey, VK_SPACE},
        WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG, WM_HOTKEY},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    ToggleLauncher,
}
const HOTKEY_ID: i32 = 1;

pub fn start() -> (thread::JoinHandle<()>, Receiver<HotkeyEvent>) {
    let (tx, rx) = unbounded();

    let h = thread::spawn(move || unsafe {
        RegisterHotKey(HWND::default(), HOTKEY_ID, MOD_ALT, VK_SPACE.0 as u32)
            .expect("Failed to register hotkey");

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            if msg.message == WM_HOTKEY {
                if tx.send(HotkeyEvent::ToggleLauncher).is_err() {
                    break;
                }
            }

            DispatchMessageW(&msg);
        }

        let _ = UnregisterHotKey(HWND::default(), 1);
    });

    (h, rx)
}
