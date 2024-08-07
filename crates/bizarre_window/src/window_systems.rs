use bizarre_ecs::system::system_param::ResMut;
use bizarre_event::EventQueue;

use crate::window_manager::WindowManager;

pub fn drain_window_events(
    mut window_manager: ResMut<WindowManager>,
    mut event_queue: ResMut<EventQueue>,
) {
    window_manager
        .drain_window_events(&mut event_queue)
        .unwrap_or_else(|err| panic!("Could not drain window events for `WindowManager`: {err}"))
}
