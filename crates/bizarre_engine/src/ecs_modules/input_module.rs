use bizarre_ecs::{system::schedule::Schedule, world::ecs_module::EcsModule};
use bizarre_event::EventQueue;
use bizarre_input::input_manager::InputManager;

use crate::prelude::*;

pub struct InputModule;

impl EcsModule for InputModule {
    fn apply(self, world: &mut bizarre_ecs::world::World) {
        world.insert_resource(InputManager::new());
        world.add_systems(Schedule::Preupdate, process_input_events);
    }
}

fn process_input_events(mut manager: ResMut<InputManager>, mut eq: ResMut<EventQueue>) {
    manager.handle_events(&mut eq);
}
