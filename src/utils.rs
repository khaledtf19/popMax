use std::path::{Path, PathBuf};

/// Config/settings directory (e.g. `%LOCALAPPDATA%/PopMax`)
pub fn get_load_path() -> Option<PathBuf> {
    let local_appdata = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local_appdata).join("PopMax"))
}

/// Directory where the executable lives — assets are shipped alongside it.
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

/// Resolve a path relative to the executable's directory.
pub fn asset_path(relative: &str) -> Option<PathBuf> {
    exe_dir().map(|d| d.join(relative))
}
