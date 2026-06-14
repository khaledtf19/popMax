use image::RgbaImage;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use widestring::U16CString;
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{BITMAPINFO, CreateCompatibleDC, DeleteDC, DeleteObject, SelectObject},
    UI::WindowsAndMessaging::{DI_NORMAL, DrawIconEx},
};
use windows::Win32::{
    Graphics::Gdi::CreateDIBSection,
    UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON},
};
use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, GetObjectW, ReleaseDC,
    },
    UI::Shell::ExtractIconExW,
};
use windows::core::PCWSTR;

#[derive(Debug)]
pub struct IconHandle(HICON);

impl IconHandle {
    pub fn raw(&self) -> HICON {
        self.0
    }
    pub fn convert(&self) -> Result<RgbaImage, String> {
        unsafe {
            // -----------------------------
            // Get icon information
            // -----------------------------
            let mut icon_info = std::mem::zeroed();

            GetIconInfo(self.0, &mut icon_info).map_err(|e| format!("GetIconInfo failed: {e}"))?;

            struct IconInfoGuard(windows::Win32::UI::WindowsAndMessaging::ICONINFO);

            impl Drop for IconInfoGuard {
                fn drop(&mut self) {
                    unsafe {
                        let _ = DeleteObject(self.0.hbmColor);
                        let _ = DeleteObject(self.0.hbmMask);
                    }
                }
            }

            let _guard = IconInfoGuard(icon_info);

            // -----------------------------
            // Get bitmap size
            // -----------------------------
            let mut bitmap = BITMAP::default();

            if GetObjectW(
                icon_info.hbmColor,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as _),
            ) == 0
            {
                return Err("GetObjectW failed".into());
            }

            let width = bitmap.bmWidth;
            let height = bitmap.bmHeight;

            // -----------------------------
            // Create DIB
            // -----------------------------
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits = std::ptr::null_mut();

            let dib = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
                .map_err(|e| format!("CreateDIBSection failed: {e}"))?;

            struct DibGuard(windows::Win32::Graphics::Gdi::HBITMAP);

            impl Drop for DibGuard {
                fn drop(&mut self) {
                    unsafe {
                        let _ = DeleteObject(self.0);
                    }
                }
            }

            let _dib_guard = DibGuard(dib);

            // -----------------------------
            // Device contexts
            // -----------------------------
            let screen = GetDC(HWND::default());

            struct DcGuard(HWND, windows::Win32::Graphics::Gdi::HDC);

            impl Drop for DcGuard {
                fn drop(&mut self) {
                    unsafe {
                        let _ = ReleaseDC(self.0, self.1);
                    }
                }
            }

            let _screen_guard = DcGuard(HWND::default(), screen);

            let mem_dc = CreateCompatibleDC(screen);

            struct MemDcGuard(windows::Win32::Graphics::Gdi::HDC);

            impl Drop for MemDcGuard {
                fn drop(&mut self) {
                    unsafe {
                        let _ = DeleteDC(self.0);
                    }
                }
            }

            let _mem_guard = MemDcGuard(mem_dc);

            // -----------------------------
            // Select bitmap
            // -----------------------------
            let old = SelectObject(mem_dc, dib);

            struct SelectGuard(
                windows::Win32::Graphics::Gdi::HDC,
                windows::Win32::Graphics::Gdi::HGDIOBJ,
            );

            impl Drop for SelectGuard {
                fn drop(&mut self) {
                    unsafe {
                        let _ = SelectObject(self.0, self.1);
                    }
                }
            }

            let _select_guard = SelectGuard(mem_dc, old);

            // -----------------------------
            // Render icon
            // -----------------------------
            DrawIconEx(mem_dc, 0, 0, self.0, width, height, 0, None, DI_NORMAL)
                .map_err(|e| format!("DrawIconEx failed: {e}"))?;

            // -----------------------------
            // Copy pixels
            // -----------------------------
            let mut buf =
                std::slice::from_raw_parts(bits as *const u8, (width * height * 4) as usize)
                    .to_vec();

            let has_alpha = buf.chunks_exact(4).any(|p| p[3] != 0);

            // BGRA -> RGBA
            for pixel in buf.chunks_exact_mut(4) {
                pixel.swap(0, 2);

                if !has_alpha {
                    pixel[3] = 255;
                }
            }

            RgbaImage::from_vec(width as u32, height as u32, buf)
                .ok_or_else(|| "Failed to create image".to_string())
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let img = self.convert()?;
        img.save(path).map_err(|e| e.to_string())
    }
}

impl Drop for IconHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

fn extract_icon_handle(path: &Path, index: i32) -> Option<IconHandle> {
    let path = path.to_str()?;
    let wide_path = U16CString::from_str(path).ok()?;

    let mut large_icon = HICON::default();

    let extracted = unsafe {
        ExtractIconExW(
            PCWSTR(wide_path.as_ptr()),
            index,
            Some(&mut large_icon),
            None,
            1,
        )
    };

    if extracted == 0 || large_icon.is_invalid() {
        return None;
    }

    Some(IconHandle(large_icon))
}
fn icon_cache_path(path: &Path, index: i32) -> Option<PathBuf> {
    let local_appdata = std::env::var_os("LOCALAPPDATA")?;

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    index.hash(&mut hasher);

    let filename = format!("{:x}.png", hasher.finish());

    Some(
        PathBuf::from(local_appdata)
            .join("PopMax")
            .join("icons")
            .join(filename),
    )
}
pub fn extract_icon(path: &Path, index: i32) -> Option<PathBuf> {
    let cache_path = icon_cache_path(path, index)?;
    if cache_path.exists() {
        return Some(cache_path);
    }

    let icon = extract_icon_handle(path, index)?;

    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).ok()?;
    }

    icon.save(&cache_path).ok()?;

    Some(cache_path)
}
