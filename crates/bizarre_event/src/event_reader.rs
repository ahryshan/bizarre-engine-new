use std::{
    any::type_name,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bizarre_ecs::{
    prelude::Resource,
    system::{system_param::SystemParam, WorldAccess, WorldAccessType},
};

use crate::{Event, EventQueue};

#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct EventReader {
    pub(crate) id: usize,
}

pub struct Events<T: Event> {
    events: Option<Box<[T]>>,
}

impl<T: Event> Deref for Events<T> {
    type Target = Option<Box<[T]>>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl<T: Event> DerefMut for Events<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

impl<T: Event> SystemParam for Events<T> {
    type Item<'w, 's> = Events<T>;

    type State = EventReader;

    unsafe fn init(world: bizarre_ecs::world::unsafe_world_cell::UnsafeWorldCell) -> Self::State {
        let eq = world
            .unsafe_world_mut()
            .resource_mut::<EventQueue>()
            .unwrap_or_else(|| {
                panic!("Cannot create an `EventReader` when there is `EventQueue` in the world")
            });

        let reader = eq.create_reader();

        eq.register_reader::<T>(reader)
            .map_err(|err| {
                panic!("Could not register an `EventReader` for `Events` system param: {err}")
            })
            .unwrap();

        reader
    }

    unsafe fn get_item<'w, 's>(
        world: bizarre_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        param_state: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        let events = world
            .resource_mut::<EventQueue>()
            .unwrap()
            .pull_events(param_state);

        Events { events }
    }

    fn param_access() -> Vec<WorldAccess> {
        vec![WorldAccess {
            resource_id: EventQueue::resource_id(),
            resource_name: EventQueue::resource_name(),
            access_type: WorldAccessType::ResWrite,
        }]
    }
}
