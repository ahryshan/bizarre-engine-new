use nalgebra_glm::IVec2;
use nalgebra_glm::Vec2;

use bizarre_core::bit_buffer::BitBuffer;
use bizarre_ecs::prelude::*;

pub use input_event::InputEvent;
pub use sdl::keyboard::Mod as Keymod;
pub use sdl::keyboard::Scancode;
pub use sdl::mouse::MouseButton;

use crate::context::with_sdl_context;

mod input_event;

#[derive(Resource)]
pub struct InputState {
    prev_keyboard_state: BitBuffer,
    keyboard_state: BitBuffer,
    keymod: Keymod,
    prev_mouse_state: BitBuffer,
    mouse_state: BitBuffer,
    mouse_position: IVec2,
    prev_mouse_position: IVec2,
    mouse_scroll_delta: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        let keyboard_state = with_sdl_context(|sdl| {
            let event_pump = sdl.event_pump().unwrap();
            let mut keyboard_state = BitBuffer::new(Scancode::Num as usize);

            let sdl_state = sdl::keyboard::KeyboardState::new(&event_pump);

            sdl_state
                .scancodes()
                .for_each(|(scancode, state)| keyboard_state.set(scancode as usize, state));

            keyboard_state
        });

        let (mouse_state, mouse_position) = with_sdl_context(|sdl| {
            let event_pump = sdl.event_pump().unwrap();
            let mut mouse_state = BitBuffer::new_short();
            let sdl_state = sdl::mouse::MouseState::new(&event_pump);
            let mouse_position = IVec2::new(sdl_state.x(), sdl_state.y());

            sdl_state
                .mouse_buttons()
                .for_each(|(button, state)| mouse_state.set(button as usize, state));

            (mouse_state, mouse_position)
        });

        Self {
            prev_keyboard_state: BitBuffer::new(keyboard_state.width()),
            keyboard_state,
            keymod: Keymod::empty(),
            prev_mouse_state: BitBuffer::new(mouse_state.width()),
            mouse_state,
            mouse_position,
            prev_mouse_position: mouse_position,
            mouse_scroll_delta: Vec2::zeros(),
        }
    }

    pub fn was_key_pressed(&self, scancode: Scancode) -> bool {
        self.prev_keyboard_state.get(scancode as usize).unwrap()
    }

    pub fn was_key_just_pressed(&self, scancode: Scancode) -> bool {
        !self.was_key_pressed(scancode) && self.is_key_pressed(scancode)
    }

    pub fn is_key_pressed(&self, scancode: Scancode) -> bool {
        self.keyboard_state.get(scancode as usize).unwrap()
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = Scancode> {
        self.keyboard_state
            .iter()
            .enumerate()
            .filter_map(|(i, pressed)| pressed.then_some(Scancode::from_i32(i as i32)?))
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_state.get(button as usize).unwrap()
    }

    pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
        self.prev_mouse_state.get(button as usize).unwrap()
    }

    pub fn keymod(&self) -> Keymod {
        self.keymod
    }

    pub fn mouse_position(&self) -> IVec2 {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> IVec2 {
        self.mouse_position - self.prev_mouse_position
    }

    pub fn process_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyPressed {
                scancode, keymod, ..
            } => {
                self.keyboard_state.set(scancode as usize, true);
                self.keymod = keymod;
            }
            InputEvent::KeyReleased {
                scancode, keymod, ..
            } => {
                self.keyboard_state.set(scancode as usize, false);
                self.keymod = keymod;
            }
            InputEvent::MouseButtonPressed { button, .. } => {
                self.mouse_state.set(button as usize, true)
            }
            InputEvent::MouseButtonReleased { button, .. } => {
                self.mouse_state.set(button as usize, false)
            }
            InputEvent::MouseMoved { pos, .. } => self.mouse_position = pos,
            InputEvent::MouseScrolled { scroll_delta, .. } => {
                self.mouse_scroll_delta += scroll_delta
            }
            _ => (),
        }
    }

    pub fn swap_frames(&mut self) {
        self.prev_keyboard_state.copy_from(&self.keyboard_state);
        self.prev_mouse_state.copy_from(&self.mouse_state);
        self.prev_mouse_position = self.mouse_position;
        self.mouse_scroll_delta = Vec2::zeros();
    }
}
