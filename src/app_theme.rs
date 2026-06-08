// maybe i will use this but not now
use gpui::{Hsla, hsla};

pub struct LauncherTheme {}

impl LauncherTheme {
    /// Main window background
    pub fn background(&self) -> Hsla {
        hsla(220.0 / 360.0, 0.16, 0.07, 1.0)
    }
    /// Search input background,
    /// Cards background,
    /// List items default background,
    pub fn surface(&self) -> Hsla {
        hsla(220.0 / 360.0, 0.20, 0.11, 1.0)
    }
    /// Mouse hover on list items,
    /// Hover on pinned apps
    pub fn surface_hover(&self) -> Hsla {
        hsla(222.0 / 360.0, 0.24, 0.15, 1.0)
    }
    /// Borders for cards and input
    pub fn border(&self) -> Hsla {
        hsla(220.0 / 360.0, 0.23, 0.21, 1.0)
    }
    /// Selected item border
    /// Focused input border
    pub fn primary(&self) -> Hsla {
        hsla(220.0 / 360.0, 1.0, 0.68, 1.0)
    }
    /// Text
    pub fn text(&self) -> Hsla {
        hsla(216.0 / 360.0, 0.29, 0.97, 1.0)
    }
    /// Subtitle text
    /// File paths
    /// Secondary labels
    pub fn text_secondary(&self) -> Hsla {
        hsla(217.0 / 360.0, 0.15, 0.73, 1.0)
    }
}
