use bizarre_window::WindowHandle;
use nalgebra_glm::Vec2;

use crate::keyboard::{Keyboard, KeyboardModifier};

#[derive(Clone, Debug)]
pub enum InputEventSource {
    Window(WindowHandle),
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    KeyPress {
        source: InputEventSource,
        modifiers: KeyboardModifier,
        key: Keyboard,
    },
    KeyRelease {
        source: InputEventSource,
        key: Keyboard,
    },
    KeyRepeat {
        source: InputEventSource,
        modifiers: KeyboardModifier,
        key: Keyboard,
    },
    PointerMove {
        source: InputEventSource,
        position: Vec2,
        delta: Vec2,
    },
}
