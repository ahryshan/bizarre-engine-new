use nalgebra_glm::{IVec2, UVec2};
use xcb::x;

use crate::window::WindowHandle;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    WindowClosed(WindowHandle),
    WindowResized {
        handle: WindowHandle,
        size: UVec2,
    },
    WindowMoved {
        handle: WindowHandle,
        position: IVec2,
    },
    MainWindowCloseRequested(WindowHandle),
}

impl WindowEvent {
    pub fn window_handle(&self) -> WindowHandle {
        use WindowEvent::*;
        match self {
            WindowClosed(handle)
            | WindowResized { handle, .. }
            | WindowMoved { handle, .. }
            | MainWindowCloseRequested(handle) => *handle,
        }
    }
}
