use std::path::PathBuf;

use crate::{
    types::{AppIcon, Item, Kind, RunCommand},
    windows_icons::extract_icon,
};
use rayon::prelude::*;

struct ScannedApp {
    name: String,
    target: String,
    icon_location: PathBuf,
    icon_index: i32,
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

            let (location, icon_index) = match lnk.string_data().icon_location().clone() {
                Some(location) => (location, *lnk.header().icon_index()),
                None => (target.clone(), 0),
            };
            // let icon_path = extract_icon(Path::new(&location), icon_index);

            result.push(ScannedApp {
                name,
                target,
                icon_location: PathBuf::from(location),
                icon_index,
            });
        }
    }
    result
}

pub fn run_scan() -> Vec<Item> {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let user_start_menu = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs", appdata);

    let programs_path = r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs";

    let dirs = [&user_start_menu, programs_path];
    let mut scanned_apps = vec![];

    for dir in dirs {
        scanned_apps.extend(scan_directory(dir));
    }
    let items = scanned_apps
        .into_par_iter()
        .map(|app| {
            let icon_path = extract_icon(&app.icon_location, app.icon_index);

            Item {
                name: app.name,
                kind: Kind::App,
                icon_path: icon_path.map(AppIcon::File),
                running_command: Some(RunCommand {
                    command: app.target,
                    args: vec![],
                }),
            }
        })
        .collect::<Vec<_>>();

    items
}
