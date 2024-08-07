use std::{collections::HashMap, sync::atomic::Ordering};

use ecs_module::EcsModule;
use unsafe_world_cell::UnsafeWorldCell;

use crate::{
    commands::command_buffer::RawCommandBuffer,
    component::{component_batch::ComponentBatch, Component, ComponentRegistry},
    entity::{Entity, EntitySpawner},
    resource::{IntoStored, Resource, ResourceId, StoredResource},
    system::{schedule::Schedule, system_config::IntoSystemConfigs, system_graph::SystemGraph},
};

pub mod ecs_module;
pub mod unsafe_world_cell;

#[derive(Default)]
pub struct World {
    pub(crate) resources: HashMap<ResourceId, StoredResource>,
    pub(crate) components: ComponentRegistry,
    pub(crate) spawner: EntitySpawner,
    pub(crate) schedules: HashMap<Schedule, SystemGraph>,
    pub(crate) deferred_commands: RawCommandBuffer,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_entity(&mut self) -> Entity {
        let (entity, reused) = self.spawner.new_entity();
        if !reused {
            self.components.expand()
        }
        self.components.register_entity(entity);
        entity
    }

    pub fn spawn_entity(&mut self, batch: impl ComponentBatch) -> Entity {
        let entity = self.create_entity();

        self.components.insert_batch(entity, batch);

        entity
    }

    pub fn kill(&mut self, entity: Entity) {
        self.spawner.kill(entity);
        self.components.remove_entity(entity);
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources
            .insert(R::resource_id(), resource.into_stored());
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources
            .remove(&R::resource_id())
            .map(|r| unsafe { r.into_inner() })
    }

    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources
            .get(&R::resource_id())
            .map(|r| unsafe { r.as_ref() })
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&R::resource_id())
            .map(|r| unsafe { r.as_mut() })
    }

    pub fn register_component<C: Component>(&mut self) {
        self.components.register::<C>()
    }

    pub fn register_components<C: ComponentBatch>(&mut self) {
        self.components.register_batch::<C>();
    }

    pub fn insert_component<C: Component>(&mut self, entity: Entity, component: C) -> Option<C> {
        self.components.insert(entity, component)
    }

    pub fn insert_components<C: ComponentBatch>(&mut self, entity: Entity, components: C) {
        self.components.insert_batch(entity, components)
    }

    pub fn component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.components.component(entity)
    }

    pub fn component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.components.component_mut(entity)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.components.remove(entity)
    }

    pub fn remove_components<C: ComponentBatch>(&mut self, entity: Entity) {
        self.components.remove_batch::<C>(entity)
    }

    pub fn add_schedule(&mut self, schedule: Schedule) {
        if self.schedules.contains_key(&schedule) {
            panic!("Trying to insert a `Schedule` {schedule:?} while there is already one in this world");
        }

        self.schedules.insert(schedule, SystemGraph::new());
    }

    pub fn init_schedule(&mut self, schedule: Schedule) {
        self.flush();

        self.with_schedule(schedule, |world, sg| sg.init_systems(world));
    }

    pub fn run_schedule(&mut self, schedule: Schedule) {
        self.flush();

        let mut cmd = self.with_schedule(schedule, |world, sg| sg.run_systems(world));
        if !cmd.is_empty() {
            unsafe { self.deferred_commands.append(&mut cmd.as_raw()) }
        }
    }

    fn with_schedule<T, F>(&mut self, schedule: Schedule, func: F) -> T
    where
        F: FnOnce(&mut World, &mut SystemGraph) -> T,
    {
        let mut sg = self.schedules.remove(&schedule).unwrap_or_else(|| {
            panic!("Trying to access `{schedule:?}` but the `World` does not have this one")
        });

        let ret = func(self, &mut sg);

        self.schedules.insert(schedule, sg);

        ret
    }

    pub fn flush(&mut self) {
        if !unsafe { self.deferred_commands.is_empty() } {
            unsafe {
                self.deferred_commands
                    .clone()
                    .apply_or_drop_queued(Some(self.into()))
            }
        }
    }

    pub fn add_systems<M>(&mut self, schedule: Schedule, systems: impl IntoSystemConfigs<M>) {
        self.with_schedule(schedule, |_, sg| sg.add_systems(systems));
    }

    pub fn add_module(&mut self, module: impl EcsModule) {
        module.apply(self);
    }

    pub unsafe fn as_unsafe_cell(&self) -> UnsafeWorldCell {
        UnsafeWorldCell::new(self)
    }

    pub fn entity_count(&self) -> u64 {
        self.spawner.next_id.load(Ordering::SeqCst) - self.spawner.dead.len() as u64
    }
}
