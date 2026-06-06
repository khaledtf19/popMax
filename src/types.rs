use std::path::PathBuf;

use gpui_component::Selectable;

#[derive(Debug, Clone)]
pub struct RunCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub kind: Kind, // maybe a Command or an App
    pub icon_path: Option<PathBuf>,
    pub running_command: Option<RunCommand>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    Command,
    App,
}
