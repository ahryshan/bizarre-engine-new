use std::marker::PhantomData;

use crate::{world::command_queue::Command, Resource};

pub struct InsertResourceCmd<T: Resource> {
    resource: T,
}

impl<T: Resource> InsertResourceCmd<T> {
    pub fn new(resource: T) -> Self {
        Self { resource }
    }
}

impl<T: Resource> Command for InsertResourceCmd<T> {
    fn apply(self, world: &mut crate::World) {
        world.insert_resource(self.resource);
    }
}

pub struct RemoveResourceCmd<T: Resource>(PhantomData<T>);

impl<T: Resource> RemoveResourceCmd<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Resource> Default for RemoveResourceCmd<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Resource> Command for RemoveResourceCmd<T> {
    fn apply(self, world: &mut crate::World) {
        world.remove_resource::<T>();
    }
}
