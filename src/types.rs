use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub icon_path: Option<PathBuf>,
    pub running_command: Option<RunCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    Command,
    App,
    /// Virtual item for a bang search shortcut (e.g. `!g query`)
    Search,
}
