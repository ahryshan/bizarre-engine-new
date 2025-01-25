use bizarre_app::app_event::AppEvent;
use bizarre_core::Handle;
use bizarre_ecs::{
    prelude::{Res, ResMut},
    system::schedule::Schedule,
    world::ecs_module::EcsModule,
};
use bizarre_event::{EventQueue, Events};
use bizarre_log::core_info;
use bizarre_sdl::{
    context::{with_sdl_context, with_sdl_events},
    input::{InputEvent, InputState},
    window::{try_handle_sdl_event, WindowCreateInfo, WindowEvent, WindowHandle, Windows},
};

use bizarre_sdl::sdl;

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
        world.insert_resource(InputState::new());
        world.add_systems(Schedule::Preupdate, (push_sdl_events, update_input_state));
    }
}

fn update_input_state(mut input: ResMut<InputState>, events: Events<InputEvent>) {
    input.swap_frames();

    for event in events {
        input.process_event(event)
    }
}

fn push_sdl_events(windows: Res<Windows>, mut event_queue: ResMut<EventQueue>) {
    with_sdl_context(|sdl| {
        sdl.event_pump()
            .unwrap()
            .poll_iter()
            .for_each(|event| try_push_event(&windows, &mut event_queue, &event));
    })
}

fn try_push_event(windows: &Windows, event_queue: &mut EventQueue, event: &SdlEvent) {
    if let Some(event) = try_handle_sdl_event(windows, event) {
        if let WindowEvent::MainWindowCloseRequested(_) = event {
            event_queue.push_event(AppEvent::CloseRequested);
        }
        event_queue.push_event(event)
    }

    if let Some(event) = InputEvent::try_from_sdl(event) {
        event_queue.push_event(event)
    }
}
