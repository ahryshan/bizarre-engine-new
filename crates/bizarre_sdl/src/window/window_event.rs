use nalgebra_glm::{IVec2, UVec2};

use super::WindowHandle;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Shown(WindowHandle),
    Hidden(WindowHandle),
    Exposed(WindowHandle),
    CloseRequested(WindowHandle),
    WindowMustClose(WindowHandle),
    MainWindowCloseRequested(WindowHandle),
    MainWindowMustClose(WindowHandle),
    Moved { handle: WindowHandle, pos: IVec2 },
    Resized { handle: WindowHandle, size: UVec2 },
    Minimized(WindowHandle),
    Maximized(WindowHandle),
    Restored(WindowHandle),
    MouseEnter(WindowHandle),
    MouseLeave(WindowHandle),
    KeyboardFocusGained(WindowHandle),
    KeyboardFocusLost(WindowHandle),
}

impl WindowEvent {
    pub fn window_handle(&self) -> WindowHandle {
        use WindowEvent::*;
        match self {
            CloseRequested(handle)
            | Resized { handle, .. }
            | Moved { handle, .. }
            | Shown(handle)
            | Hidden(handle)
            | Exposed(handle)
            | WindowMustClose(handle)
            | MainWindowCloseRequested(handle)
            | MainWindowMustClose(handle)
            | Minimized(handle)
            | Maximized(handle)
            | Restored(handle)
            | MouseEnter(handle)
            | MouseLeave(handle)
            | KeyboardFocusGained(handle)
            | KeyboardFocusLost(handle) => *handle,
        }
    }
}
