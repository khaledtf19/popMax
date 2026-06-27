use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::PathBuf, thread};

use widestring::u16cstr;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
                NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
                DispatchMessageW, GetCursorPos, GetMessageW, HICON, HMENU, IDI_APPLICATION,
                IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, LoadIconW, LoadImageW, MF_STRING, MSG,
                PostMessageW, RegisterClassW, SetForegroundWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON,
                TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP,
                WM_CONTEXTMENU, WM_DESTROY, WM_NULL, WM_RBUTTONUP, WNDCLASSW,
            },
        },
    },
    core::PCWSTR,
};

const TRAY_ID: u32 = 1;
const TRAY_CALLBACK: u32 = WM_APP + 1;
const EXIT_MENU_ID: usize = 1001;

pub fn start() {
    thread::spawn(|| {
        if let Err(err) = unsafe { run_tray() } {
            eprintln!("Failed to start tray icon: {err}");
        }
    });
}

unsafe fn run_tray() -> windows::core::Result<()> {
    let class_name = u16cstr!("PopMax.TrayWindow");
    let window_name = u16cstr!("PopMax Tray");

    let window_class = WNDCLASSW {
        lpfnWndProc: Some(tray_wnd_proc),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    unsafe {
        RegisterClassW(&window_class);
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            HWND::default(),
            HMENU::default(),
            None,
            None,
        )?
    };

    unsafe {
        add_tray_icon(hwnd)?;
    }

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        delete_tray_icon(hwnd);
    }

    Ok(())
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        TRAY_CALLBACK if wparam.0 as u32 == TRAY_ID => {
            let event = lparam.0 as u32;
            if event == WM_CONTEXTMENU || event == WM_RBUTTONUP {
                unsafe {
                    show_context_menu(hwnd);
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe {
                delete_tray_icon(hwnd);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn add_tray_icon(hwnd: HWND) -> windows::core::Result<()> {
    let icon = unsafe { load_tray_icon()? };
    let mut data = tray_data(hwnd);

    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_CALLBACK;
    data.hIcon = icon;
    write_tip(&mut data.szTip, "PopMax");

    unsafe {
        Shell_NotifyIconW(NIM_ADD, &data).ok()?;
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &data).ok()?;
    }

    Ok(())
}

unsafe fn load_tray_icon() -> windows::core::Result<HICON> {
    for path in tray_icon_paths() {
        let wide_path = wide_path(&path);
        if let Ok(handle) = unsafe {
            LoadImageW(
                None,
                PCWSTR(wide_path.as_ptr()),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            )
        } {
            return Ok(HICON(handle.0));
        }
    }

    unsafe { LoadIconW(None, IDI_APPLICATION) }
}

fn tray_icon_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            paths.push(exe_dir.join("PopMaxIcon.ico"));
        }
    }

    paths.push(PathBuf::from("installer").join("PopMaxIcon.ico"));

    paths
}

fn wide_path(path: &PathBuf) -> Vec<u16> {
    OsStr::new(path).encode_wide().chain(Some(0)).collect()
}

unsafe fn delete_tray_icon(hwnd: HWND) {
    let data = tray_data(hwnd);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    let Ok(menu) = (unsafe { CreatePopupMenu() }) else {
        return;
    };

    let exit_label = u16cstr!("Exit");
    let _ = unsafe { AppendMenuW(menu, MF_STRING, EXIT_MENU_ID, PCWSTR(exit_label.as_ptr())) };

    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor).is_err() } {
        unsafe {
            let _ = DestroyMenu(menu);
        }
        return;
    }

    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }

    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            cursor.x,
            cursor.y,
            0,
            hwnd,
            None,
        )
    };

    // PostMessage WM_NULL is required after SetForegroundWindow + TrackPopupMenu
    // to ensure the context menu behaves correctly. Without this call, the menu
    // may appear and immediately dismiss or not appear at all.
    unsafe {
        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
    }

    unsafe {
        let _ = DestroyMenu(menu);
    }

    if command.0 as usize == EXIT_MENU_ID {
        unsafe {
            delete_tray_icon(hwnd);
        }
        std::process::exit(0);
    }
}

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

fn write_tip(target: &mut [u16; 128], text: &str) {
    for (dest, source) in target.iter_mut().zip(text.encode_utf16()) {
        *dest = source;
    }
}
