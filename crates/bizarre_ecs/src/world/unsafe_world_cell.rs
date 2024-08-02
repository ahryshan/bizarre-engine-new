use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::resource::{Resource, ResourceId};

use super::World;

#[derive(Clone, Copy)]
pub struct UnsafeWorldCell<'w>(*mut World, PhantomData<&'w World>);

impl<'w> UnsafeWorldCell<'w> {
    pub fn new(world: &World) -> Self {
        Self(ptr::from_ref(world).cast_mut(), PhantomData)
    }

    unsafe fn unsafe_world(self) -> &'w World {
        &*self.0
    }

    unsafe fn unsafe_world_mut(self) -> &'w mut World {
        &mut *self.0
    }

    pub fn resource<R: Resource>(self) -> Option<&'w R> {
        unsafe {
            self.unsafe_world()
                .resources
                .get(&R::id())
                .map(|r| r.as_ref())
        }
    }

    pub fn resource_mut<R: Resource>(self) -> Option<&'w mut R> {
        unsafe {
            self.unsafe_world_mut()
                .resources
                .get_mut(&R::id())
                .map(|r| r.as_mut())
        }
    }
}
