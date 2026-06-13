use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub kind: Kind, // maybe a Command or an App
    pub icon_path: Option<AppIcon>,
    pub running_command: Option<RunCommand>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    Command,
    App,
}

#[derive(Debug, Clone)]
pub enum AppIcon {
    File(PathBuf), // .png, .ico, etc.
    WindowsResource {
        path: PathBuf, // .exe or .dll
        index: i32,
    },
}
