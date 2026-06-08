use crate::types::{Item, Kind, RunCommand};

fn scan_directory(path: &str) -> Vec<Item> {
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

            if !target.ends_with(".exe") {
                continue;
            }

            let name = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            result.push(Item {
                name: name,
                kind: Kind::App,
                icon_path: None,
                running_command: Some(RunCommand {
                    command: target,
                    args: vec![],
                }),
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
    let mut res = vec![];

    for dir in dirs {
        res.extend(scan_directory(dir));
    }

    res
}
