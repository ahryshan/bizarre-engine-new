use std::marker::PhantomData;

use crate::{world::command_queue::Command, Component, Entity, World};

pub struct AddComponentCmd<T>
where
    T: Component,
{
    entity: Entity,
    component: T,
}

impl<T> AddComponentCmd<T>
where
    T: Component,
{
    pub fn new(entity: Entity, component: T) -> Self {
        Self { entity, component }
    }
}

impl<T> Command for AddComponentCmd<T>
where
    T: Component,
{
    fn apply(self, world: &mut World) {
        world.register_component::<T>();
        world.insert_component(self.entity, self.component);
    }
}

pub struct RemoveComponentCmd<T>
where
    T: Component,
{
    entity: Entity,
    _marker: PhantomData<T>,
}

impl<T: Component> RemoveComponentCmd<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

impl<T: Component> Command for RemoveComponentCmd<T> {
    fn apply(self, world: &mut World) {
        world.remove_component::<T>(self.entity);
    }
}

pub struct RegisterComponentCmd<T: Component>(PhantomData<T>);

impl<T: Component> RegisterComponentCmd<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> Default for RegisterComponentCmd<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> Command for RegisterComponentCmd<T> {
    fn apply(self, world: &mut World) {
        world.register_component::<T>();
    }
}
