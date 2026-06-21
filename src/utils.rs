use std::path::PathBuf;

pub fn get_load_path() -> Option<PathBuf> {
    let local_appdata = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local_appdata).join("PopMax"))
}
