use nalgebra_glm::IVec2;
use nalgebra_glm::Vec2;

use super::Keymod;
use super::MouseButton;
use super::Scancode;
use crate::window::WindowHandle;

use sdl::event::Event as SdlEvent;

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed {
        window: WindowHandle,
        scancode: Scancode,
        keymod: Keymod,
    },
    KeyReleased {
        window: WindowHandle,
        scancode: Scancode,
        keymod: Keymod,
    },
    MouseButtonPressed {
        window: WindowHandle,
        button: MouseButton,
        pos: IVec2,
    },
    MouseDoubleClick {
        window: WindowHandle,
        button: MouseButton,
        pos: IVec2,
    },
    MouseButtonReleased {
        window: WindowHandle,
        button: MouseButton,
        pos: IVec2,
    },
    MouseMoved {
        window: WindowHandle,
        pos: IVec2,
    },
    MouseScrolled {
        window: WindowHandle,
        scroll_delta: Vec2,
    },
}

impl InputEvent {
    pub fn try_from_sdl(event: &SdlEvent) -> Option<InputEvent> {
        match event {
            SdlEvent::KeyDown {
                window_id,
                scancode,
                keymod,
                repeat,
                ..
            } if !repeat => Some(InputEvent::KeyPressed {
                window: WindowHandle::from_raw(*window_id as usize),
                scancode: *scancode.as_ref()?,
                keymod: *keymod,
            }),
            SdlEvent::KeyUp {
                window_id,
                scancode,
                keymod,
                repeat,
                ..
            } if !repeat => Some(InputEvent::KeyReleased {
                window: WindowHandle::from_raw(*window_id as usize),
                scancode: *scancode.as_ref()?,
                keymod: *keymod,
            }),
            SdlEvent::MouseButtonDown {
                window_id,
                mouse_btn,
                x,
                y,
                ..
            } => Some(InputEvent::MouseButtonPressed {
                window: WindowHandle::from_raw(*window_id as usize),
                button: *mouse_btn,
                pos: IVec2::new(*x, *y),
            }),
            SdlEvent::MouseButtonUp {
                window_id,
                mouse_btn,
                x,
                y,
                ..
            } => Some(InputEvent::MouseButtonReleased {
                window: WindowHandle::from_raw(*window_id as usize),
                button: *mouse_btn,
                pos: IVec2::new(*x, *y),
            }),
            SdlEvent::MouseMotion {
                window_id, x, y, ..
            } => Some(InputEvent::MouseMoved {
                pos: IVec2::new(*x, *y),
                window: WindowHandle::from_raw(*window_id as usize),
            }),
            SdlEvent::MouseWheel {
                window_id,
                precise_x,
                precise_y,
                ..
            } => Some(InputEvent::MouseScrolled {
                window: WindowHandle::from_raw(*window_id as usize),
                scroll_delta: Vec2::new(*precise_x, *precise_y),
            }),
            _ => None,
        }
    }
}
