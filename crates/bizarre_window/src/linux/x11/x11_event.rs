use nalgebra_glm::{IVec2, UVec2};
use xcb::x;

use crate::window::WindowHandle;

#[derive(Clone)]
pub enum X11WindowEvent {
    DestroyNotify {
        handle: WindowHandle,
    },
    ConfigureNotify {
        handle: WindowHandle,
        position: IVec2,
        size: UVec2,
    },
    ClientMessage {
        handle: WindowHandle,
        data: x::ClientMessageData,
    },
}

impl X11WindowEvent {
    pub fn window_handle(&self) -> WindowHandle {
        use X11WindowEvent::*;

        match self {
            DestroyNotify { handle, .. }
            | ConfigureNotify { handle, .. }
            | ClientMessage { handle, .. } => *handle,
        }
    }
}
