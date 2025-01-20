use bizarre_core::bit_vec::BitVec;

pub use input_event::InputEvent;
pub use sdl::keyboard::Mod as Keymod;
pub use sdl::keyboard::Scancode;
pub use sdl::mouse::MouseButton;

use crate::context::with_sdl_context;

mod input_event;

pub struct InputState {
    keyboard_state: BitVec,
    mouse_state: BitVec,
}

impl InputState {
    pub fn new() -> Self {
        let keyboard_state = with_sdl_context(|sdl| {
            let event_pump = sdl.event_pump().unwrap();
            let mut keyboard_state = BitVec::new((Scancode::Num as usize / 8) + 1);
            sdl::keyboard::KeyboardState::new(&event_pump)
                .scancodes()
                .for_each(|(scancode, state)| keyboard_state.set_bit(scancode as usize, state));

            keyboard_state
        });

        let mouse_state = with_sdl_context(|sdl| {
            let event_pump = sdl.event_pump().unwrap();
            let mut mouse_state = BitVec::new_short();
            sdl::mouse::MouseState::new(&event_pump)
                .mouse_buttons()
                .for_each(|(button, state)| mouse_state.set_bit(button as usize, state));
            mouse_state
        });

        Self {
            keyboard_state,
            mouse_state,
        }
    }

    pub fn is_keyboard_down(&self, scancode: Scancode) -> bool {
        self.keyboard_state.get(scancode as usize).unwrap()
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_state.get(button as usize).unwrap()
    }

    pub fn process_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyPressed { scancode, .. } => {
                self.keyboard_state.set_bit(scancode as usize, true)
            }
            InputEvent::KeyReleased { scancode, .. } => {
                self.keyboard_state.set_bit(scancode as usize, false)
            }
            _ => {}
        }
    }
}
