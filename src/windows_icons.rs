use image::RgbaImage;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use widestring::U16CString;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON};
use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, GetDIBits, GetObjectW, ReleaseDC,
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
    pub unsafe fn convert(&self) -> Result<RgbaImage, String> {
        unsafe {
            let mut icon_info = std::mem::zeroed();
            if !GetIconInfo(self.0, &mut icon_info).is_ok() {
                return Err("Failed to get icon info".into());
            }

            let mut bitmap: BITMAP = std::mem::zeroed();
            let bitmap_size = std::mem::size_of::<BITMAP>() as i32;
            if GetObjectW(
                icon_info.hbmColor,
                bitmap_size,
                Some(&mut bitmap as *mut BITMAP as *mut _),
            ) == 0
            {
                return Err("Failed to get bitmap object".into());
            }

            let width = bitmap.bmWidth as u32;
            let height = bitmap.bmHeight as u32;
            let buf_size = (width * height * 4) as usize;
            let mut buf = vec![0u8; buf_size];

            let mut bi = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bitmap.bmWidth,
                biHeight: -bitmap.bmHeight, // Top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };

            let hdc = GetDC(HWND::default());
            if GetDIBits(
                hdc,
                icon_info.hbmColor,
                0,
                height,
                Some(buf.as_mut_ptr() as *mut _),
                &mut bi as *mut BITMAPINFOHEADER as *mut _,
                DIB_RGB_COLORS,
            ) == 0
            {
                ReleaseDC(HWND::default(), hdc);
                return Err("Failed to extract DIB bits".into());
            }
            ReleaseDC(HWND::default(), hdc);

            // Convert BGRA (windows default) to RGBA (Image crate)
            for chunk in buf.chunks_mut(4) {
                chunk.swap(0, 2); // B and R
            }
            // Clean up GDI handles from GetIconInfo
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(icon_info.hbmMask);
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(icon_info.hbmColor);

            match RgbaImage::from_vec(width, height, buf) {
                Some(img) => Ok(img),
                None => Err("Failed to create RgbaImage from raw buffer".into()),
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        unsafe {
            let img = self.convert()?;
            img.save(path).map_err(|e| e.to_string())
        }
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
