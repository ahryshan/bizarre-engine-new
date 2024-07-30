use crate::{
    component::component_cmd::{AddComponentCmd, RegisterComponentCmd, RemoveComponentCmd},
    entity::entity_commands::{KillEntitiesCmd, SpawnEntityCmd},
    query::query_element::QueryElement,
    resource::resource_cmd::{InsertResourceCmd, RemoveResourceCmd},
    Component, Entity, Resource,
};

use super::command_queue::{Command, CommandQueue};

#[derive(Default)]
pub struct Commands {
    queue: CommandQueue,
}

impl Commands {
    pub fn into_queue(self) -> CommandQueue {
        self.queue
    }

    pub fn kill_entities(&mut self, entities: &[Entity]) {
        self.queue.push(KillEntitiesCmd {
            entities: entities.into(),
        })
    }

    pub fn create_entity(&mut self) {
        self.queue.push(SpawnEntityCmd::default());
    }

    pub fn entity_with<F>(&mut self, f: F)
    where
        F: FnOnce(&mut SpawnEntityCmd),
    {
        let mut cmd = SpawnEntityCmd::new();

        f(&mut cmd);

        self.queue.push(cmd);
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.queue.push(AddComponentCmd::new(entity, component));
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) {
        self.queue.push(RemoveComponentCmd::<C>::new(entity))
    }

    pub fn register_component<C: Component>(&mut self) {
        self.queue.push(RegisterComponentCmd::<C>::new())
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.queue.push(InsertResourceCmd::<R>::new(resource))
    }

    pub fn remove_resource<R: Resource>(&mut self) {
        self.queue.push(RemoveResourceCmd::<R>::new())
    }

    pub fn push_custom_cmd<T: Command>(&mut self, command: T) {
        self.queue.push(command)
    }
}
