use nalgebra_glm::{IVec2, UVec2, Vec2};

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
        keycode: u8,
    },
    KeyRelease {
        handle: WindowHandle,
        keycode: u8,
    },
    KeyboardModifierChange {
        handle: WindowHandle,
    },
    PointerMove {
        handle: WindowHandle,
        position: Vec2,
    },
    ButtonPress {
        handle: WindowHandle,
        button: u8,
    },
    ButtonRelease {
        handle: WindowHandle,
        button: u8,
    },
    Scroll {
        handle: WindowHandle,
        delta: Vec2,
    },
    GainedKeyboardFocus(WindowHandle),
    LostKeyboardFocus(WindowHandle),
    GainedMouseFocus(WindowHandle),
    LostMouseFocus(WindowHandle),
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
            | KeyboardModifierChange { handle, .. }
            | PointerMove { handle, .. }
            | ButtonPress { handle, .. }
            | ButtonRelease { handle, .. }
            | Scroll { handle, .. }
            | GainedKeyboardFocus(handle)
            | LostKeyboardFocus(handle)
            | MainWindowCloseRequest(handle)
            | GainedMouseFocus(handle)
            | LostMouseFocus(handle) => *handle,
        }
    }
}
