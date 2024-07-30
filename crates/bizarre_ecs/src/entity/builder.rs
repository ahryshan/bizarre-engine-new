use crate::{component::component_storage::IntoStoredComponent, world::World};

use super::entity_commands::SpawnEntityCmd;

pub struct EntityBuilder<'a> {
    world: &'a mut World,
    cmd: SpawnEntityCmd,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self {
            world,
            cmd: Default::default(),
        }
    }

    #[must_use = "build() must be called for entity to spawn"]
    pub fn with_component<C: IntoStoredComponent>(mut self, component: C) -> Self {
        self.cmd.components.push(component.into_stored_component());
        self
    }

    pub fn build(self) {
        self.world.push_command(self.cmd);
    }
}
