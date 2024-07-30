use crate::{
    component::component_storage::{IntoStoredComponent, StoredComponent},
    world::command_queue::Command,
    Entity, World,
};

#[derive(Default)]
pub struct SpawnEntityCmd {
    pub components: Vec<StoredComponent>,
}

impl Command for SpawnEntityCmd {
    fn apply(self, world: &mut World) {
        let entity = world.create_entity();

        for comp in self.components {
            world
                .components
                .register_raw(comp.inner_type_id(), comp.component_name());
            world.insert_component(entity, comp);
        }
    }
}

impl SpawnEntityCmd {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_component<C: IntoStoredComponent>(&mut self, component: C) -> &mut Self {
        self.components.push(component.into_stored_component());
        self
    }
}

pub struct KillEntitiesCmd {
    pub entities: Vec<Entity>,
}

impl Command for KillEntitiesCmd {
    fn apply(self, world: &mut World) {
        for entity in self.entities {
            world.kill(entity).unwrap();
        }
    }
}
