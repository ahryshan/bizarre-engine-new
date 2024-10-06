use nalgebra_glm::{IVec2, UVec2};

use crate::WindowMode;

/// Window create info
///
/// These properties does not guarantee the proper setup: [size](WindowCreateInfo::size),
/// [position](WindowCreateInfo::position). This behaviour depends on underlying API and Window
/// Manager
pub struct WindowCreateInfo {
    /// Size hint for the underlying API. There is no guarantee, that
    /// window will be created with the specified size, and that depends on the
    /// API and window manager.
    /// In case of creating window with [WindowMode::Fullscreen] of `maximized = true`,
    /// the specified size will be used for the windowed mode of the window in case of
    /// transition to normal mode.
    pub size: UVec2,
    /// Position hint for the underlying API. There is no guarantee, that
    /// window will be created with the specified position, and that depends on the
    /// API and window manager
    pub position: IVec2,
    /// Window title
    pub title: String,
    pub mode: WindowMode,
    pub maximized: bool,
}

impl WindowCreateInfo {
    pub fn normal_window(title: String, size: UVec2) -> Self {
        Self {
            maximized: false,
            mode: WindowMode::Windowed,
            position: [0, 0].into(),
            title,
            size,
        }
    }

    pub fn no_border_window(title: String, size: UVec2) -> Self {
        Self {
            maximized: false,
            mode: WindowMode::WindowedBorderless,
            position: [0, 0].into(),
            title,
            size,
        }
    }

    pub fn splash_window(title: String, size: UVec2) -> Self {
        Self {
            maximized: false,
            mode: WindowMode::WindowedBorderless,
            size,
            title,
            position: [0, 0].into(),
        }
    }

    pub fn fullscreen_window(title: String) -> Self {
        Self {
            mode: WindowMode::Fullscreen,
            ..Self::normal_window(title, [600, 400].into())
        }
    }
}
