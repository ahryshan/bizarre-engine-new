use std::collections::HashMap;

use unsafe_world_cell::UnsafeWorldCell;

use crate::{
    component::{Component, ComponentRegistry},
    entity::{Entity, EntitySpawner},
    resource::{IntoStored, Resource, ResourceId, Stored},
};

pub mod unsafe_world_cell;

#[derive(Default)]
pub struct World {
    pub(crate) resources: HashMap<ResourceId, Stored>,
    pub(crate) components: ComponentRegistry,
    pub(crate) spawner: EntitySpawner,
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

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(R::id(), resource.into_stored());
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources
            .remove(&R::id())
            .map(|r| unsafe { r.into_inner() })
    }

    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get(&R::id()).map(|r| unsafe { r.as_ref() })
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&R::id())
            .map(|r| unsafe { r.as_mut() })
    }

    pub fn register_component<C: Component>(&mut self) {
        self.components.register::<C>()
    }

    pub fn insert_component<C: Component>(&mut self, entity: Entity, component: C) -> Option<C> {
        self.components.insert(entity, component)
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

    pub unsafe fn as_unsafe_cell(&self) -> UnsafeWorldCell {
        UnsafeWorldCell::new(self)
    }
}
