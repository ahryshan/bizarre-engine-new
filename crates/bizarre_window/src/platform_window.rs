use bizarre_event::EventQueue;
use nalgebra_glm::{IVec2, UVec2};

use crate::{window_error::WindowResult, WindowCreateInfo, WindowHandle, WindowMode, WindowStatus};

pub trait PlatformWindow {
    fn new(create_info: &WindowCreateInfo) -> WindowResult<Self>
    where
        Self: Sized;

    /// Returns the internally saved size
    fn size(&self) -> UVec2;

    /// Returns the internally saved position
    fn position(&self) -> IVec2;

    fn mode(&self) -> WindowMode;
    fn raw_handle(&self) -> u64;
    fn handle(&self) -> WindowHandle;
    fn title(&self) -> &str;

    fn status(&self) -> WindowStatus;

    fn set_size(&mut self, size: UVec2) -> WindowResult<()>;

    /// Sets position of the window. Does not have any effect on Wayland when applied to TopLevel
    fn set_position(&mut self, position: IVec2) -> WindowResult<()>;
    fn set_mode(&mut self, mode: WindowMode) -> WindowResult<()>;
    fn set_title(&mut self, title: String) -> WindowResult<()>;
    fn minimize(&mut self) -> WindowResult<()>;
    fn maximize(&mut self) -> WindowResult<()>;
    fn unmaximize(&mut self) -> WindowResult<()>;
    fn toggle_maximize(&mut self) -> WindowResult<()>;

    /// Processes events from underlying windowing system and from the window itself.
    /// In most cases it will push new events to the provided [EventQueue]
    fn process_events(&mut self, event_queue: &mut EventQueue) -> WindowResult<()>;

    fn close(&mut self) -> WindowResult<()>;
    fn close_requested(&self) -> bool;
}
