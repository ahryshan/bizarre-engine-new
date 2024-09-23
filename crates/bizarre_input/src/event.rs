use bizarre_event::Event;
use bizarre_window::WindowHandle;

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
}
