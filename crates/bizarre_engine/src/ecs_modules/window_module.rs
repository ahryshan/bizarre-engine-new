use crate::prelude::*;

use bizarre_app::app_event::AppEvent;
use bizarre_ecs::{
    system::schedule::Schedule,
    world::{ecs_module::EcsModule, World},
};
use bizarre_event::EventQueue;
use bizarre_window::{window_events::WindowEvent, window_manager::WindowManager, WindowCreateInfo};

pub struct WindowModule {
    windows: Vec<(bool, WindowCreateInfo)>,
}

impl WindowModule {
    pub fn new() -> Self {
        Self {
            windows: Default::default(),
        }
    }

    pub fn with_window(mut self, create_info: WindowCreateInfo, main_window: bool) -> Self {
        self.windows.push((main_window, create_info));
        self
    }
}

impl EcsModule for WindowModule {
    fn apply(self, world: &mut World) {
        let mut window_manager = WindowManager::new();

        for (main_window, create_info) in self.windows {
            let (h, _) = window_manager.create_window(&create_info).unwrap();

            if main_window {
                window_manager.set_main_window(h);
            }
        }

        world.insert_resource(window_manager);
        world.add_systems(Schedule::Preupdate, process_window_events);
    }
}

pub fn process_window_events(
    mut window_manager: ResMut<WindowManager>,
    mut event_queue: ResMut<EventQueue>,
) {
    window_manager
        .drain_window_events(&mut event_queue)
        .unwrap_or_else(|err| panic!("Could not drain window events for `WindowManager`: {err}"));

    window_manager
        .iter_mut()
        .for_each(|(_, w)| w.process_events(&mut event_queue).unwrap());

    if let Some(close_requested) = window_manager
        .get_main_window()
        .map(|w| w.close_requested())
    {
        if close_requested {
            // TODO: Make system for tracking if app should close
            // event_queue.push_event(WindowEvent::MainWindowCloseRequested(
            //     window_manager.get_main_window_handle(),
            // ));

            event_queue.push_event(AppEvent::CloseRequested);
        }
    }
}
