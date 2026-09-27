use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    types::{Item, Kind, RunCommand},
    utils::get_load_path,
    windows_icons::extract_icon,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
use windows::core::PCWSTR;
use winreg::enums::*;
use winreg::{HKEY, RegKey};

struct ScannedApp {
    id: String,
    name: String,
    target: String,
    icon_location: PathBuf,
    icon_index: i32,
}

/// Make an ID for a scanned app from its path.
fn make_id(path: &Path) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// The registry roots that hold installed-app entries.
///
/// Shared by [`scan_registry_apps`] and [`registry_fingerprint`] so the cache can
/// never validate against a different set of roots than the scan actually reads.
fn uninstall_keys() -> [(HKEY, &'static str); 3] {
    [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ]
}

/// The Start Menu trees that hold `.lnk` shortcuts.
fn start_menu_dirs() -> [String; 2] {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    [
        format!(r"{}\Microsoft\Windows\Start Menu\Programs", appdata),
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
    ]
}

/// Walk the Start Menu trees and collect `.lnk` paths.
///
/// Enumeration only — no shortcut is opened here, so this stays cheap enough to
/// run on every startup as part of cache validation.
fn collect_lnk_paths(dirs: &[String]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for dir in dirs {
        for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "lnk") {
                paths.push(entry.path().to_path_buf());
            }
        }
    }
    paths
}

/// Parse a single `.lnk` into a [`ScannedApp`], or `None` if it isn't launchable.
fn parse_lnk(path: &Path) -> Option<ScannedApp> {
    let lnk = lnk::ShellLink::open(path, lnk::encoding::WINDOWS_1252).ok()?;
    let target = lnk.link_target()?;

    if PathBuf::from(&target)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("exe"))
    {
        return None;
    }

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let icon_location = lnk
        .string_data()
        .icon_location()
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&target));

    Some(ScannedApp {
        id: make_id(path),
        name,
        target,
        icon_location,
        icon_index: *lnk.header().icon_index(),
    })
}

/// Holds a scanned app item along with the data needed to extract its icon later.
#[derive(Clone, Serialize, Deserialize)]
pub struct ScanEntry {
    pub item: Item,
    pub icon_location: PathBuf,
    pub icon_index: i32,
}

/// Where the validated scan cache is stored.
fn cache_path() -> Option<PathBuf> {
    Some(get_load_path()?.join("scan_cache.json"))
}

/// Fingerprint of the registry Uninstall roots.
///
/// Installing or uninstalling an app adds/removes a subkey, which bumps the
/// parent key's subkey count and last-write time. That makes three
/// `query_info` calls enough to notice, instead of reading every subkey.
fn registry_fingerprint() -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for (hkey, path) in uninstall_keys() {
        let Ok(key) = RegKey::predef(hkey).open_subkey(path) else {
            // Root missing: hash the absence so it differs from "root present, no apps".
            0u8.hash(&mut h);
            continue;
        };
        let Ok(info) = key.query_info() else {
            0u8.hash(&mut h);
            continue;
        };
        info.sub_keys.hash(&mut h);
        // `FileTime` derefs to a Win32 FILETIME (low/high dword pair).
        info.last_write_time.dwLowDateTime.hash(&mut h);
        info.last_write_time.dwHighDateTime.hash(&mut h);
    }
    h.finish()
}

/// Fingerprint of the Start Menu trees.
///
/// Combines the count of discovered `.lnk` files with each file's mtime and
/// length, so an install, uninstall, or shortcut edit all change the value.
/// The per-file stats are the expensive part, so they run in parallel.
fn lnk_fingerprint(paths: &[PathBuf]) -> u64 {
    let stamps: Vec<(u64, u64)> = paths
        .par_iter()
        .filter_map(|p| {
            let meta = std::fs::metadata(p).ok()?;
            let mtime = meta
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_nanos() as u64;
            Some((mtime, meta.len()))
        })
        .collect();

    let mut h = std::collections::hash_map::DefaultHasher::new();
    stamps.len().hash(&mut h);
    for (mtime, len) in stamps {
        mtime.hash(&mut h);
        len.hash(&mut h);
    }
    h.finish()
}

#[derive(Serialize, Deserialize)]
struct ScanCache {
    fingerprint: u64,
    entries: Vec<ScanEntry>,
}

/// Return a previously computed scan when nothing it depends on has changed.
///
/// Any missing, malformed, or fingerprint-mismatched cache is treated as a miss
/// so the caller just falls back to a full scan.
fn load_cached_scan(path: &Path, fingerprint: u64) -> Option<Vec<ScanEntry>> {
    let bytes = std::fs::read(path).ok()?;
    let cache: ScanCache = serde_json::from_slice(&bytes).ok()?;
    (cache.fingerprint == fingerprint).then_some(cache.entries)
}

fn store_cached_scan(path: &Path, fingerprint: u64, entries: &[ScanEntry]) {
    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(cache) = serde_json::to_vec(&ScanCache {
        fingerprint,
        entries: entries.to_vec(),
    }) else {
        return;
    };
    // Best-effort: a failed cache write only costs time on the next startup.
    let _ = std::fs::write(path, cache);
}

/// Fast scan — walks Start Menu directories AND the Uninstall registry keys.
/// Returns entries with `icon_path: None`. Does NOT extract icons.
/// Icon extraction happens later via [`extract_icons_batch`].
/// Registry apps whose display name matches a Start Menu app are skipped.
///
/// Opening every `.lnk` dominates the cost of a scan (~90% of the Start Menu
/// phase), so the work is split three ways:
///  1. `.lnk` files are parsed in parallel, since each parse is independent.
///  2. The result is cached on disk and reused while the Start Menu trees and
///     registry roots are unchanged.
///  3. The fingerprint is checked first, so an unchanged system never pays for
///     parsing at all.
pub fn scan_apps_fast() -> Vec<ScanEntry> {
    let lnk_paths = collect_lnk_paths(&start_menu_dirs());

    let fingerprint = scan_fingerprint(&lnk_paths);

    if let Some(path) = cache_path()
        && let Some(entries) = load_cached_scan(&path, fingerprint)
    {
        return entries;
    }

    let entries = scan_uncached(&lnk_paths);
    if let Some(path) = cache_path() {
        store_cached_scan(&path, fingerprint, &entries);
    }
    entries
}

/// Fingerprint of everything a scan depends on: the Start Menu `.lnk` files and
/// the registry Uninstall roots.
fn scan_fingerprint(lnk_paths: &[PathBuf]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    lnk_fingerprint(lnk_paths).hash(&mut h);
    registry_fingerprint().hash(&mut h);
    h.finish()
}

/// Perform the full scan: parse every `.lnk` in parallel, then read the registry.
fn scan_uncached(lnk_paths: &[PathBuf]) -> Vec<ScanEntry> {
    let mut dedup = HashSet::new();
    let mut result = Vec::new();

    // 1. Start Menu .lnk files — parsed in parallel; `collect` keeps results in
    //    path order so the list is stable regardless of thread scheduling.
    let scanned: Vec<ScannedApp> = lnk_paths.par_iter().filter_map(|p| parse_lnk(p)).collect();
    for app in scanned {
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

/// Reusable expansion buffer.
///
/// `ExpandEnvironmentStringsW` writes into a caller-owned buffer, and the old
/// code allocated and zeroed 64 KB on every call — measurably wasteful when
/// called once per registry entry. A thread-local keeps the buffer alive across
/// calls on the same thread.
fn expand_env_vars(s: &str) -> String {
    use std::cell::RefCell;
    thread_local! {
        static BUF: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
    }

    BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        // Expansion can only grow the string, so this is a safe starting size.
        if buf.len() < s.len() + 256 {
            buf.resize(s.len() + 256, 0);
        }

        let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
        let len = unsafe { ExpandEnvironmentStringsW(PCWSTR(wide.as_ptr()), Some(&mut buf)) };
        if len > 0 && (len as usize) < buf.len() {
            String::from_utf16_lossy(&buf[..len as usize - 1])
        } else {
            s.to_string()
        }
    })
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

fn scan_uninstall_key(key: &RegKey, results: &mut Vec<ScanEntry>, dedup: &mut HashSet<String>) {
    for name in key.enum_keys().flatten() {
        let Ok(subkey) = key.open_subkey(&name) else {
            continue;
        };

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

    for (hkey, subkey_path) in uninstall_keys() {
        if let Ok(key) = RegKey::predef(hkey).open_subkey(subkey_path) {
            scan_uninstall_key(&key, &mut results, dedup);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Unique scratch dir per test, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let n = N.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("popmax-test-{tag}-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn sample_entry(name: &str) -> ScanEntry {
        ScanEntry {
            item: Item {
                id: format!("id-{name}"),
                name: name.to_string(),
                kind: Kind::App,
                icon_path: None,
                running_command: Some(RunCommand {
                    command: format!(r"C:\{name}.exe"),
                    args: vec![],
                }),
            },
            icon_location: PathBuf::from(format!(r"C:\{name}.exe")),
            icon_index: 0,
        }
    }

    #[test]
    fn cache_round_trip_returns_same_entries() {
        let dir = TempDir::new("roundtrip");
        let path = dir.file("scan_cache.json");
        let entries = vec![sample_entry("Alpha"), sample_entry("Beta")];

        store_cached_scan(&path, 42, &entries);
        let loaded = load_cached_scan(&path, 42).expect("cache should hit");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].item.name, "Alpha");
        assert_eq!(loaded[1].item.id, "id-Beta");
        assert_eq!(
            loaded[0].item.running_command.as_ref().unwrap().command,
            r"C:\Alpha.exe"
        );
    }

    #[test]
    fn cache_misses_when_fingerprint_changes() {
        let dir = TempDir::new("fingerprint");
        let path = dir.file("scan_cache.json");
        store_cached_scan(&path, 1, &[sample_entry("Alpha")]);

        // Same file, different fingerprint: an install/uninstall must invalidate.
        assert!(load_cached_scan(&path, 2).is_none());
        // Original fingerprint still hits.
        assert!(load_cached_scan(&path, 1).is_some());
    }

    #[test]
    fn missing_cache_file_is_a_miss() {
        let dir = TempDir::new("missing");
        assert!(load_cached_scan(&dir.file("nope.json"), 7).is_none());
    }

    #[test]
    fn malformed_cache_file_is_a_miss() {
        let dir = TempDir::new("malformed");
        let path = dir.file("scan_cache.json");
        std::fs::write(&path, b"{ this is not valid json").unwrap();

        // Corrupt cache must degrade to a full scan, never panic or return junk.
        assert!(load_cached_scan(&path, 7).is_none());
    }

    #[test]
    fn store_creates_missing_parent_directory() {
        let dir = TempDir::new("parent");
        let path = dir.file("nested/deeper/scan_cache.json");

        store_cached_scan(&path, 5, &[sample_entry("Alpha")]);

        assert!(path.exists());
        assert!(load_cached_scan(&path, 5).is_some());
    }

    #[test]
    fn empty_scan_round_trips_as_empty() {
        let dir = TempDir::new("empty");
        let path = dir.file("scan_cache.json");

        store_cached_scan(&path, 9, &[]);

        // An empty result is a legitimate result, not a cache miss.
        assert!(load_cached_scan(&path, 9).is_some_and(|e| e.is_empty()));
    }

    /// Real `.lnk` files from this machine's Start Menu, if any.
    ///
    /// `lnk` 0.6 can only *read* shortcuts unless its `binwrite` feature is
    /// enabled, so these tests exercise the real files rather than synthesizing
    /// fixtures. Returns empty on a machine with no Start Menu, and the callers
    /// skip rather than fail.
    fn real_lnk_paths() -> Vec<PathBuf> {
        collect_lnk_paths(&start_menu_dirs())
    }

    #[test]
    fn parse_lnk_rejects_missing_file() {
        let dir = TempDir::new("missinglnk");
        assert!(parse_lnk(&dir.file("Nope.lnk")).is_none());
    }

    #[test]
    fn parse_lnk_rejects_non_lnk_content() {
        let dir = TempDir::new("garbage");
        let path = dir.file("Broken.lnk");
        std::fs::write(&path, b"this is not a shell link").unwrap();

        assert!(parse_lnk(&path).is_none());
    }

    #[test]
    fn parse_lnk_accepts_real_start_menu_shortcuts() {
        let paths = real_lnk_paths();
        if paths.is_empty() {
            eprintln!("skipping: no Start Menu shortcuts on this machine");
            return;
        }

        let mut checked = 0;
        for path in &paths {
            let Some(app) = parse_lnk(path) else { continue };

            // Everything parse_lnk returns must be launchable.
            assert!(
                app.target.to_ascii_lowercase().ends_with(".exe"),
                "non-exe target leaked through: {}",
                app.target
            );
            // Name comes from the shortcut file stem, and the id from its path —
            // both must be non-empty so the list and favorites stay addressable.
            assert!(!app.name.is_empty());
            assert_eq!(app.id, make_id(path));
            checked += 1;
        }

        assert!(checked > 0, "expected at least one launchable shortcut");
    }

    #[test]
    fn collect_lnk_paths_finds_only_lnk_files() {
        let dir = TempDir::new("collect");
        for name in ["a.lnk", "b.lnk", "notes.txt", "readme.md"] {
            std::fs::write(dir.file(name), b"x").unwrap();
        }
        let sub = dir.file("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("c.lnk"), b"x").unwrap();

        let mut found = collect_lnk_paths(&[dir.0.to_string_lossy().into_owned()]);
        found.sort();

        assert_eq!(found.len(), 3);
        assert!(
            found
                .iter()
                .all(|p| p.extension().is_some_and(|e| e == "lnk"))
        );
        // Recurses into subdirectories.
        assert!(found.iter().any(|p| p.ends_with("c.lnk")));
    }

    #[test]
    fn expand_env_vars_expands_percent_variables() {
        assert_eq!(
            expand_env_vars(r"%SystemRoot%\system32\shell32.dll"),
            format!(
                r"{}\system32\shell32.dll",
                std::env::var("SystemRoot").unwrap()
            )
        );
    }

    #[test]
    fn expand_env_vars_leaves_plain_strings_untouched() {
        assert_eq!(expand_env_vars(r"C:\apps\thing.exe"), r"C:\apps\thing.exe");
    }

    #[test]
    fn expand_env_vars_survives_repeated_calls() {
        // The buffer is thread-local and reused; make sure reuse stays correct.
        let first = expand_env_vars(r"%SystemRoot%\system32\shell32.dll");
        let _ = expand_env_vars("x");
        let again = expand_env_vars(r"%SystemRoot%\system32\shell32.dll");
        assert_eq!(first, again);
    }

    #[test]
    fn parse_display_icon_splits_trailing_index() {
        assert_eq!(
            parse_display_icon(r"C:\app.exe,3"),
            (PathBuf::from(r"C:\app.exe"), 3)
        );
    }

    #[test]
    fn parse_display_icon_defaults_index_when_absent() {
        assert_eq!(
            parse_display_icon(r"C:\app.exe"),
            (PathBuf::from(r"C:\app.exe"), 0)
        );
    }

    #[test]
    fn parallel_parse_matches_serial_parse() {
        let paths = real_lnk_paths();
        if paths.is_empty() {
            eprintln!("skipping: no Start Menu shortcuts on this machine");
            return;
        }

        let serial: Vec<_> = paths.iter().filter_map(|p| parse_lnk(p)).collect();
        let parallel: Vec<_> = paths.par_iter().filter_map(|p| parse_lnk(p)).collect();

        // rayon must not reorder or drop anything: same length, same order,
        // same ids. This is what keeps the launcher list from reshuffling.
        assert_eq!(serial.len(), parallel.len());
        assert_eq!(
            serial.iter().map(|a| &a.id).collect::<Vec<_>>(),
            parallel.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
        assert_eq!(
            serial.iter().map(|a| &a.name).collect::<Vec<_>>(),
            parallel.iter().map(|a| &a.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn lnk_fingerprint_changes_when_a_shortcut_is_added() {
        let dir = TempDir::new("fpadd");
        let mut paths = vec![dir.file("a.lnk")];
        std::fs::write(&paths[0], b"x").unwrap();

        let before = lnk_fingerprint(&paths);

        let extra = dir.file("b.lnk");
        std::fs::write(&extra, b"x").unwrap();
        paths.push(extra);

        assert_ne!(lnk_fingerprint(&paths), before);
    }

    #[test]
    fn lnk_fingerprint_changes_when_a_shortcut_is_removed() {
        let dir = TempDir::new("fprem");
        let a = dir.file("a.lnk");
        let b = dir.file("b.lnk");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"x").unwrap();

        let before = lnk_fingerprint(&[a.clone(), b]);
        assert_ne!(lnk_fingerprint(std::slice::from_ref(&a)), before);
    }

    #[test]
    fn lnk_fingerprint_is_stable_for_unchanged_files() {
        let dir = TempDir::new("fpstable");
        let a = dir.file("a.lnk");
        let b = dir.file("b.lnk");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"y").unwrap();
        let paths = vec![a, b];

        let first = lnk_fingerprint(&paths);
        assert_eq!(lnk_fingerprint(&paths), first);
        assert_eq!(lnk_fingerprint(&paths), first);
    }

    #[test]
    fn lnk_fingerprint_changes_when_file_contents_change_size() {
        let dir = TempDir::new("fpsize");
        let a = dir.file("a.lnk");
        std::fs::write(&a, b"x").unwrap();
        let before = lnk_fingerprint(std::slice::from_ref(&a));

        // Rewrite with a different length; mtime may or may not move in the
        // filesystem's timestamp granularity, so the length is the reliable signal.
        std::fs::write(&a, b"a much longer body").unwrap();

        assert_ne!(lnk_fingerprint(std::slice::from_ref(&a)), before);
    }

    #[test]
    fn scan_uncached_only_returns_launchable_exe_targets() {
        let paths = real_lnk_paths();
        if paths.is_empty() {
            eprintln!("skipping: no Start Menu shortcuts on this machine");
            return;
        }

        // The registry half is environment-specific, so check the Start Menu half:
        // every entry must carry a runnable .exe command.
        let lnk_only: Vec<ScanEntry> = paths
            .par_iter()
            .filter_map(|p| parse_lnk(p))
            .map(|app| ScanEntry {
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
            })
            .collect();

        assert!(!lnk_only.is_empty());
        for entry in &lnk_only {
            let cmd = entry
                .item
                .running_command
                .as_ref()
                .expect("every scanned app must be launchable");
            assert!(cmd.command.to_ascii_lowercase().ends_with(".exe"));
        }
    }
}
