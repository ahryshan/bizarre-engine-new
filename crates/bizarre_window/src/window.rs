use nalgebra_glm::{IVec2, UVec2};

use crate::WindowCreateInfo;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum WindowMode {
    Fullscreen,
    #[default]
    Windowed,
}

pub struct WindowStatus {
    pub minimized: bool,
    pub maximized: bool,
    pub mapped: bool,
}

pub trait WindowTrait {
    fn new(create_info: &WindowCreateInfo) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Returns the internally saved size. The returned value may be stale, so in order
    /// to retrieve the exact and actual values it's recommended to
    /// use [WindowTrait::update_size_and_position]
    fn size(&self) -> UVec2;

    /// Returns the internally saved position. The returned value may be stale, so in order
    /// to retrieve the exact and actual values it's recommended to
    /// use [WindowTrait::update_size_and_position]
    fn position(&self) -> IVec2;

    /// Gets the exact size and position of the window from underlying API
    /// and returns those new values.
    /// May update the internally kept size and position with those new values.
    ///
    /// Returns `(size, position)`
    fn update_size_and_position(&mut self) -> anyhow::Result<(UVec2, IVec2)>;

    fn mode(&self) -> WindowMode;
    fn raw_handle(&self) -> u32;
    fn title(&self) -> &str;

    fn status(&self) -> WindowStatus;

    fn set_size(&mut self, size: UVec2) -> anyhow::Result<()>;
    fn set_position(&mut self, position: IVec2) -> anyhow::Result<()>;
    fn set_mode(&mut self, mode: WindowMode) -> anyhow::Result<()>;
    fn set_title(&mut self, title: String) -> anyhow::Result<()>;
    fn set_decorations(&mut self, decorations: bool) -> anyhow::Result<()>;
    fn map(&mut self) -> anyhow::Result<()>;
    fn unmap(&mut self) -> anyhow::Result<()>;
    fn minimize(&mut self) -> anyhow::Result<()>;
    fn restore(&mut self) -> anyhow::Result<()>;
    fn maximize(&mut self) -> anyhow::Result<()>;
    fn unmaximize(&mut self) -> anyhow::Result<()>;
}
