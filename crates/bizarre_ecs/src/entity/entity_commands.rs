use std::marker::PhantomData;

use crate::{
    commands::{command_buffer::CommandBuffer, Command},
    component::{component_batch::ComponentBatch, component_commands::RegisterComponentsCmd},
    world::World,
};

use super::Entity;

pub struct SpawnEntityCmd<T: ComponentBatch> {
    pub components: T,
}

impl<T: ComponentBatch> SpawnEntityCmd<T> {
    pub fn new(components: T) -> Self {
        Self { components }
    }
}

impl<T: ComponentBatch> Command for SpawnEntityCmd<T> {
    fn apply(self, world: &mut World) {
        world.spawn_entity(self.components);
    }
}

pub struct InsertComponentsCmd<T: ComponentBatch> {
    pub components: T,
    pub entity: Entity,
}

impl<T: ComponentBatch> InsertComponentsCmd<T> {
    pub fn new(entity: Entity, components: T) -> Self {
        Self { entity, components }
    }
}

impl<T: ComponentBatch> Command for InsertComponentsCmd<T> {
    fn apply(self, world: &mut World) {
        world.insert_components(self.entity, self.components)
    }
}

pub struct RemoveComponentsCmd<T: ComponentBatch> {
    pub(crate) entity: Entity,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T: ComponentBatch> RemoveComponentsCmd<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

impl<T: ComponentBatch> Command for RemoveComponentsCmd<T> {
    fn apply(self, world: &mut World) {
        world.remove_components::<T>(self.entity)
    }
}

#[derive(Debug)]
pub struct KillEntity {
    entity: Entity,
}

impl KillEntity {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl Command for KillEntity {
    fn apply(self, world: &mut World) {
        world.kill(self.entity)
    }
}

#[must_use = "`EntityCmdBuilder` won't have any effect unless `build` is called"]
#[deny(unused)]
pub struct EntityCmdBuilder<'a, const IS_KILLING: bool> {
    buffer: &'a mut CommandBuffer,
    entity: Entity,
    inner_cmd: CommandBuffer,
}

impl<'a> EntityCmdBuilder<'a, false> {
    pub fn new(buffer: &'a mut CommandBuffer, entity: Entity) -> Self {
        Self {
            buffer,
            entity,
            inner_cmd: Default::default(),
        }
    }

    pub fn insert_components<T: ComponentBatch>(mut self, components: T) -> Self {
        self.inner_cmd.push(RegisterComponentsCmd::<T>::new());
        self.inner_cmd
            .push(InsertComponentsCmd::new(self.entity, components));
        self
    }

    pub fn remove_components<T: ComponentBatch>(mut self) -> Self {
        self.inner_cmd
            .push(RemoveComponentsCmd::<T>::new(self.entity));
        self
    }

    /// Kills the entity
    ///
    /// If this function is called, builder will produce only one command: [`KillEntityCmd`]. There
    /// will be no components added or removed even if there were called `add/remove_components` on
    /// this particular builder
    pub fn kill(self) -> EntityCmdBuilder<'a, true> {
        EntityCmdBuilder { ..self }
    }

    pub fn build(mut self) {
        self.buffer.append(&mut self.inner_cmd)
    }
}

impl<'a> EntityCmdBuilder<'a, true> {
    pub fn build(self) {
        self.buffer.push(KillEntity::new(self.entity))
    }
}
