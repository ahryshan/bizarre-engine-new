use std::marker::PhantomData;

use crate::{commands::Command, world::World};

use super::Resource;

pub struct InsertResourceCmd<T: Resource> {
    resource: T,
}

impl<T: Resource> InsertResourceCmd<T> {
    pub fn new(resource: T) -> Self {
        Self { resource }
    }
}

impl<T: Resource> Command for InsertResourceCmd<T> {
    fn apply(self, world: &mut World) {
        world.insert_resource(self.resource)
    }
}

pub struct RemoveResourceCmd<T: Resource> {
    _phantom: PhantomData<T>,
}

impl<T: Resource> RemoveResourceCmd<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Resource> Default for RemoveResourceCmd<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Resource> Command for RemoveResourceCmd<T> {
    fn apply(self, world: &mut World) {
        world.remove_resource::<T>();
    }
}
