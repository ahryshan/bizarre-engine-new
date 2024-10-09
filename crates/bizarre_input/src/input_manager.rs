use bizarre_ecs::prelude::*;
use bizarre_event::{EventQueue, EventReader};
use bizarre_log::core_trace;
use bizarre_window::window_events::WindowEvent;
use nalgebra_glm::Vec2;

use crate::{
    event::{InputEvent, InputEventSource},
    keyboard::{Keyboard, KeyboardModifier},
    mouse::MouseButton,
};

const KEY_COUNT: usize = 256;
const BUTTON_COUNT: usize = 32;

#[derive(Resource)]
pub struct InputManager {
    keys: [bool; KEY_COUNT],
    prev_keys: [bool; KEY_COUNT],
    keyboard_modifiers: KeyboardModifier,

    buttons: [bool; BUTTON_COUNT],
    prev_buttons: [bool; BUTTON_COUNT],

    pointer_position: Vec2,
    prev_pointer_position: Vec2,

    event_reader: Option<EventReader>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            keys: [false; _],
            prev_keys: [false; _],
            keyboard_modifiers: KeyboardModifier::None,
            buttons: [false; _],
            prev_buttons: [false; _],
            pointer_position: Vec2::zeros(),
            prev_pointer_position: Vec2::zeros(),
            event_reader: None,
        }
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn key_pressed(&self, key: Keyboard) -> bool {
        self.keys[key.as_usize()]
    }

    pub fn intersects_modifiers(&self, modifiers: KeyboardModifier) -> bool {
        self.keyboard_modifiers.intersects(modifiers)
    }

    pub fn modifiers_exact(&self, modifiers: KeyboardModifier) -> bool {
        self.keyboard_modifiers & modifiers == modifiers
    }

    pub fn key_just_pressed(&self, key: Keyboard) -> bool {
        self.keys[key.as_usize()] && !self.prev_keys[key.as_usize()]
    }

    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.buttons[button.as_usize()]
    }

    pub fn button_just_pressed(&self, button: MouseButton) -> bool {
        self.buttons[button.as_usize()] && !self.prev_buttons[button.as_usize()]
    }

    pub fn pointer_delta(&self) -> Vec2 {
        self.pointer_position - self.prev_pointer_position
    }

    pub fn pointer_position(&self) -> Vec2 {
        self.pointer_position
    }

    pub fn handle_events(&mut self, eq: &mut EventQueue) {
        let reader = self.event_reader.get_or_insert_with(|| {
            let reader = eq.create_reader();
            eq.register_reader::<WindowEvent>(reader)
                .expect("Could not register an event reader for input_manager");
            reader
        });

        if let Some(events) = eq.pull_events::<WindowEvent>(reader) {
            events
                .iter()
                .map(|ev| match ev {
                    WindowEvent::KeyPress { handle, keycode } => {
                        let key = Keyboard::from_raw(*keycode);
                        self.keyboard_modifiers |= key.into();
                        self.keys[key.as_usize()] = true;
                        let input_event = InputEvent::KeyPress {
                            key,
                            modifiers: self.keyboard_modifiers,
                            source: InputEventSource::Window(*handle),
                        };
                        Some(input_event)
                    }
                    WindowEvent::KeyRelease { handle, keycode } => {
                        let key = Keyboard::from_raw(*keycode);

                        self.keyboard_modifiers &= !KeyboardModifier::from(key);
                        self.keys[key.as_usize()] = false;

                        Some(InputEvent::KeyRelease {
                            key,
                            source: InputEventSource::Window(*handle),
                        })
                    }
                    WindowEvent::MouseMove { handle, position } => {
                        self.pointer_position = *position;

                        Some(InputEvent::PointerMove {
                            source: InputEventSource::Window(*handle),
                            position: *position,
                            delta: self.pointer_delta(),
                        })
                    }
                    _ => None,
                })
                .flatten()
                .for_each(|ev| eq.push_event(ev))
        }
    }

    pub fn change_frames(&mut self) {
        std::mem::swap(&mut self.keys, &mut self.prev_keys);
        std::mem::swap(&mut self.buttons, &mut self.prev_buttons);
        self.prev_pointer_position = self.pointer_position;
    }
}
