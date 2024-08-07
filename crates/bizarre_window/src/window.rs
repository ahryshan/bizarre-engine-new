use anyhow::Result;
use bizarre_core::Handle;
use bizarre_event::EventQueue;
use nalgebra_glm::{IVec2, UVec2};

use crate::{window_error::WindowResult, window_events::WindowEvent, Window, WindowCreateInfo};

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

pub type WindowHandle = Handle<Window>;

pub trait WindowTrait {
    fn new(create_info: &WindowCreateInfo) -> WindowResult<Self>
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
    fn update_size_and_position(&mut self) -> WindowResult<(UVec2, IVec2)>;

    fn mode(&self) -> WindowMode;
    fn raw_handle(&self) -> u32;
    fn handle(&self) -> WindowHandle;
    fn title(&self) -> &str;

    fn status(&self) -> WindowStatus;

    fn set_size(&mut self, size: UVec2) -> WindowResult<()>;
    fn set_position(&mut self, position: IVec2) -> WindowResult<()>;
    fn set_mode(&mut self, mode: WindowMode) -> WindowResult<()>;
    fn set_title(&mut self, title: String) -> WindowResult<()>;
    fn set_decorations(&mut self, decorations: bool) -> WindowResult<()>;
    fn map(&mut self) -> WindowResult<()>;
    fn unmap(&mut self) -> WindowResult<()>;
    fn minimize(&mut self) -> WindowResult<()>;
    fn restore(&mut self) -> WindowResult<()>;
    fn maximize(&mut self) -> WindowResult<()>;
    fn unmaximize(&mut self) -> WindowResult<()>;

    fn handle_events(&mut self, event_queue: &mut EventQueue) -> WindowResult<()>;

    fn close_requested(&self) -> bool;
}
