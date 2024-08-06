use std::{
    marker::PhantomData,
    ptr::{self},
};

use crate::{
    component::Component,
    entity::Entity,
    resource::{Resource, ResourceId},
};

use super::World;

#[derive(Clone, Copy)]
pub struct UnsafeWorldCell<'w>(*mut World, PhantomData<&'w World>);

impl<'w> UnsafeWorldCell<'w> {
    pub fn new(world: &World) -> Self {
        Self(ptr::from_ref(world).cast_mut(), PhantomData)
    }

    pub unsafe fn unsafe_world(self) -> &'w World {
        &*self.0
    }

    pub unsafe fn unsafe_world_mut(self) -> &'w mut World {
        &mut *self.0
    }

    pub fn resource<R: Resource>(self) -> Option<&'w R> {
        unsafe {
            self.unsafe_world()
                .resources
                .get(&R::resource_id())
                .map(|r| r.as_ref())
        }
    }

    pub fn resource_mut<R: Resource>(self) -> Option<&'w mut R> {
        unsafe {
            self.unsafe_world_mut()
                .resources
                .get_mut(&R::resource_id())
                .map(|r| r.as_mut())
        }
    }

    pub fn component<C: Component>(self, entity: Entity) -> Option<&'w C> {
        unsafe { self.unsafe_world().component(entity) }
    }

    pub fn component_mut<C: Component>(self, entity: Entity) -> Option<&'w mut C> {
        unsafe { self.unsafe_world_mut().component_mut(entity) }
    }

    pub fn filter_entities(self, ids: &[ResourceId]) -> Vec<Entity> {
        unsafe { self.unsafe_world() }
            .components
            .filter_entities(ids)
    }
}
