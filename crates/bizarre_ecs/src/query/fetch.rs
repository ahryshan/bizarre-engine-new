use std::marker::PhantomData;

use crate::{component::Component, entity::Entity, world::World};

use super::query_element::QueryElement;

pub struct Fetch<'q, T>(PhantomData<&'q T>)
where
    T: Component;

impl<'q, T> QueryElement<'q> for Fetch<'q, T>
where
    T: Component,
{
    type Item = &'q T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        Some(T::inner_type_id())
    }

    fn get_item(world: &'q World, entity: Entity) -> Self::Item {
        world.components.get::<T>(entity).unwrap()
    }
}

pub struct FetchMut<'q, T>(PhantomData<&'q T>)
where
    T: Component;

impl<'q, T> QueryElement<'q> for FetchMut<'q, T>
where
    T: Component,
{
    type Item = &'q mut T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        Some(T::inner_type_id())
    }

    fn get_item(world: &'q World, entity: Entity) -> Self::Item {
        world.components.get_mut::<T>(entity).unwrap()
    }
}
