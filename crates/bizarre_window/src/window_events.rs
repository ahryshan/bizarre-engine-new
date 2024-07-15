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

    #[cfg(feature = "x11")]
    X11ConfigureNotify {
        handle: WindowHandle,
        position: IVec2,
        size: UVec2,
    },

    #[cfg(feature = "x11")]
    X11ClientMessage {
        handle: WindowHandle,
        data: x::ClientMessageData,
    },
}

impl WindowEvent {
    pub fn window_handle(&self) -> WindowHandle {
        use WindowEvent::*;
        match self {
            WindowClosed(handle) | WindowResized { handle, .. } | WindowMoved { handle, .. } => {
                *handle
            }

            #[cfg(feature = "x11")]
            X11ConfigureNotify { handle, .. } | X11ClientMessage { handle, .. } => *handle,
        }
    }
}
