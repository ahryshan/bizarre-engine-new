use nalgebra_glm::{IVec2, UVec2};

use crate::window::WindowHandle;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Close(WindowHandle),
    Resize {
        handle: WindowHandle,
        size: UVec2,
    },
    Moved {
        handle: WindowHandle,
        position: IVec2,
    },
    MainWindowCloseRequest(WindowHandle),
    KeyPress {
        handle: WindowHandle,
        keycode: usize,
    },
    KeyRelease {
        handle: WindowHandle,
        keycode: usize,
    },
}

impl WindowEvent {
    pub fn window_handle(&self) -> WindowHandle {
        use WindowEvent::*;
        match self {
            Close(handle)
            | Resize { handle, .. }
            | Moved { handle, .. }
            | KeyPress { handle, .. }
            | KeyRelease { handle, .. }
            | MainWindowCloseRequest(handle) => *handle,
        }
    }
}
