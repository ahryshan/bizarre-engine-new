use bizarre_app::app_event::AppEvent;
use bizarre_core::Handle;
use bizarre_ecs::{prelude::ResMut, system::schedule::Schedule, world::ecs_module::EcsModule};
use bizarre_event::EventQueue;
use bizarre_sdl::{
    context::{with_sdl_context, with_sdl_events},
    window::{WindowCreateInfo, WindowHandle, Windows},
};

use bizarre_sdl::sdl;

use bizarre_window::window_events::WindowEvent;
use nalgebra_glm::UVec2;
use sdl::event::Event as SdlEvent;

pub struct SdlModule {
    windows: Vec<(bool, WindowCreateInfo)>,
}

impl SdlModule {
    pub fn new() -> Self {
        Self {
            windows: Default::default(),
        }
    }

    pub fn with_window(mut self, create_info: WindowCreateInfo) -> Self {
        self.windows.push((false, create_info));
        self
    }

    pub fn with_main_window(mut self, create_info: WindowCreateInfo) -> Self {
        self.windows.push((true, create_info));
        self
    }
}

impl EcsModule for SdlModule {
    fn apply(self, world: &mut bizarre_ecs::world::World) {
        let mut windows = Windows::new();

        for (main_window, create_info) in self.windows {
            let handle = windows.create_window(&create_info);
            if main_window {
                windows.set_main_window(handle);
            }
        }

        world.insert_resource(windows);
        world.add_systems(Schedule::Preupdate, handle_sdl_events);
    }
}

fn handle_sdl_events(mut event_queue: ResMut<EventQueue>) {
    with_sdl_context(|sdl| {
        sdl.event_pump()
            .unwrap()
            .poll_iter()
            .for_each(|event| try_push_event(&mut event_queue, event));
    })
}

fn try_push_event(event_queue: &mut EventQueue, event: SdlEvent) {
    use sdl::event::WindowEvent as SdlWindowEvent;

    match event {
        SdlEvent::Quit { .. } => event_queue.push_event(AppEvent::CloseRequested),
        SdlEvent::Window {
            window_id,
            win_event,
            ..
        } => match win_event {
            SdlWindowEvent::Resized(width, height) => event_queue.push_event(WindowEvent::Resize {
                handle: Handle::<bizarre_window::Window>::from_raw(window_id as usize),
                size: UVec2::new(width as u32, height as u32),
            }),
            _ => {}
        },
        _ => {}
    }
}
