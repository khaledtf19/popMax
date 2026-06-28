use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::{
    types::{Item, Kind, RunCommand},
    windows_icons::extract_icon,
};
use rayon::prelude::*;
use winreg::enums::*;
use winreg::RegKey;
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
use windows::core::PCWSTR;

struct ScannedApp {
    id: String,
    name: String,
    target: String,
    icon_location: PathBuf,
    icon_index: i32,
}

/// Make an ID for a scanned app from its path.
fn make_id(path: &std::path::Path) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn scan_directory(path: &str) -> Vec<ScannedApp> {
    let mut result = Vec::new();
    for entry in walkdir::WalkDir::new(path) {
        let Ok(entry) = entry else { continue };
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "lnk") {
            let Ok(lnk) = lnk::ShellLink::open(entry.path(), lnk::encoding::WINDOWS_1252) else {
                continue;
            };

            let Some(target) = lnk.link_target() else {
                continue;
            };

            if PathBuf::from(&target)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_none_or(|ext| !ext.eq_ignore_ascii_case("exe"))
            {
                continue;
            }

            let name = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            let icon_path = lnk
                .string_data()
                .icon_location()
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&target));

            let icon_index = *lnk.header().icon_index();

            let id = make_id(entry.path());

            result.push(ScannedApp {
                id,
                name,
                target,
                icon_location: icon_path,
                icon_index,
            });
        }
    }
    result
}

/// Holds a scanned app item along with the data needed to extract its icon later.
pub struct ScanEntry {
    pub item: Item,
    pub icon_location: PathBuf,
    pub icon_index: i32,
}

/// Fast scan — walks Start Menu directories AND the Uninstall registry keys.
/// Returns entries with `icon_path: None`. Does NOT extract icons.
/// Icon extraction happens later via [`extract_icons_batch`].
/// Registry apps whose display name matches a Start Menu app are skipped.
pub fn scan_apps_fast() -> Vec<ScanEntry> {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let user_start_menu = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs", appdata);

    let programs_path = r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs";

    let mut dedup = HashSet::new();
    let mut result = vec![];

    // 1. Start Menu .lnk files
    for dir in [&user_start_menu, programs_path] {
        for app in scan_directory(dir) {
            dedup.insert(app.name.to_lowercase());
            result.push(ScanEntry {
                item: Item {
                    id: app.id,
                    name: app.name,
                    kind: Kind::App,
                    icon_path: None,
                    running_command: Some(RunCommand {
                        command: app.target,
                        args: vec![],
                    }),
                },
                icon_location: app.icon_location,
                icon_index: app.icon_index,
            });
        }
    }

    // 2. Registry Uninstall keys — dedup by name against Start Menu
    result.extend(scan_registry_apps(&mut dedup));

    result
}

/// Extract icons for all entries in parallel using rayon.
/// Returns a vector of `(item_id, cached_icon_path)` pairs.
/// Entries whose icon could not be extracted return `None`.
pub fn extract_icons_batch(entries: &[ScanEntry]) -> Vec<(String, Option<PathBuf>)> {
    entries
        .par_iter()
        .map(|entry| {
            let icon_path = extract_icon(&entry.icon_location, entry.icon_index);
            (entry.item.id.clone(), icon_path)
        })
        .collect()
}

fn expand_env_vars(s: &str) -> String {
    let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 32768];
    let len = unsafe { ExpandEnvironmentStringsW(PCWSTR(wide.as_ptr()), Some(&mut buf)) };
    if len > 0 && (len as usize) < buf.len() {
        String::from_utf16_lossy(&buf[..len as usize - 1])
    } else {
        s.to_string()
    }
}

/// Parse a `DisplayIcon` registry value like `C:\app.exe,0` or
/// `%SystemRoot%\system32\imageres.dll,-12`.
fn parse_display_icon(value: &str) -> (PathBuf, i32) {
    let expanded = expand_env_vars(value);

    // If the last comma is followed by a parseable integer, split there.
    if let Some(comma) = expanded.rfind(',') {
        let after = &expanded[comma + 1..].trim();
        if let Ok(index) = after.parse::<i32>() {
            return (PathBuf::from(&expanded[..comma].trim()), index);
        }
    }
    (PathBuf::from(expanded.trim()), 0)
}

fn make_registry_id(display_icon: &str, install_location: &str, name: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    display_icon.hash(&mut h);
    install_location.hash(&mut h);
    name.hash(&mut h);
    format!("reg-{:016x}", h.finish())
}

fn scan_uninstall_key(
    key: &RegKey,
    results: &mut Vec<ScanEntry>,
    dedup: &mut HashSet<String>,
) {
    for name in key.enum_keys().flatten() {
        let Ok(subkey) = key.open_subkey(&name) else { continue };

        // ---- filter out non-app entries ----
        let Ok(display_name): Result<String, _> = subkey.get_value("DisplayName") else {
            continue;
        };
        let display_name = display_name.trim().to_string();
        if display_name.is_empty() {
            continue;
        }

        // Skip system components / updates
        let is_system: u32 = subkey.get_value("SystemComponent").unwrap_or(0);
        if is_system != 0 {
            continue;
        }
        let has_parent: Result<String, _> = subkey.get_value("ParentDisplayName");
        if has_parent.is_ok() {
            continue;
        }

        // ---- resolve launch target ----
        let display_icon_val: Result<String, _> = subkey.get_value("DisplayIcon");
        let install_loc_val: Result<String, _> = subkey.get_value("InstallLocation");

        let launch_target = resolve_launch_target(
            display_icon_val.as_deref().ok(),
            install_loc_val.as_deref().ok(),
            &display_name,
        );

        let Some(launch_target) = launch_target else {
            continue; // nothing we can launch
        };

        // ---- dedup by display name ----
        let lower = display_name.to_lowercase();
        if !dedup.insert(lower) {
            continue;
        }

        // ---- icon source ----
        let (icon_location, icon_index) = display_icon_val
            .as_deref()
            .ok()
            .map(|v| parse_display_icon(v))
            .unwrap_or_else(|| (PathBuf::from(&launch_target), 0));

        let id = make_registry_id(
            display_icon_val.as_deref().unwrap_or(""),
            install_loc_val.as_deref().unwrap_or(""),
            &display_name,
        );

        results.push(ScanEntry {
            item: Item {
                id,
                name: display_name,
                kind: Kind::App,
                icon_path: None,
                running_command: Some(RunCommand {
                    command: launch_target,
                    args: vec![],
                }),
            },
            icon_location,
            icon_index,
        });
    }
}

fn resolve_launch_target(
    display_icon: Option<&str>,
    install_location: Option<&str>,
    name: &str,
) -> Option<String> {
    // 1. DisplayIcon that points to an .exe
    if let Some(icon) = display_icon {
        let (path, _) = parse_display_icon(icon);
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
        {
            return path.to_str().map(|s| s.to_string());
        }
    }

    // 2. InstallLocation – probe for a matching .exe
    if let Some(loc) = install_location {
        let loc_path = PathBuf::from(expand_env_vars(loc));
        if loc_path.is_dir() {
            // Try: `<name>.exe`, `<name_no_spaces>.exe`, then first .exe found
            let candidates = [
                format!("{}.exe", name),
                format!("{}.exe", name.replace(' ', "")),
            ];
            for exe_name in &candidates {
                let p = loc_path.join(exe_name);
                if p.is_file() {
                    return p.to_str().map(|s| s.to_string());
                }
            }
            // Fallback: first exe in directory
            if let Ok(entries) = std::fs::read_dir(&loc_path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && p.extension()
                            .and_then(|e| e.to_str())
                            .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
                    {
                        return p.to_str().map(|s| s.to_string());
                    }
                }
            }
        }
    }

    None
}

fn scan_registry_apps(dedup: &mut HashSet<String>) -> Vec<ScanEntry> {
    let mut results = Vec::new();

    let paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (hkey, subkey_path) in &paths {
        if let Ok(key) = RegKey::predef(*hkey).open_subkey(subkey_path) {
            scan_uninstall_key(&key, &mut results, dedup);
        }
    }

    results
}
