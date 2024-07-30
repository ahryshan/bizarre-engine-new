use std::{marker::PhantomData, ptr::NonNull};

use crate::{component::Component, entity::Entity, world::World};

use super::query_element::QueryElement;

pub struct Fetch<T>(PhantomData<T>)
where
    T: Component;

impl<T> QueryElement for Fetch<T>
where
    T: Component,
{
    type Item<'a> = &'a T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        Some(T::inner_type_id())
    }

    fn get_item<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        world.components.get::<T>(entity).unwrap()
    }

    fn is_non_component() -> bool {
        false
    }
}

pub struct FetchMut<T>(PhantomData<T>)
where
    T: Component;

impl<T> QueryElement for FetchMut<T>
where
    T: Component,
{
    type Item<'a> = &'a mut T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        Some(T::inner_type_id())
    }

    fn get_item<'a>(world: &'a World, entity: Entity) -> Self::Item<'a> {
        world.components.get_mut::<T>(entity).unwrap()
    }

    fn is_non_component() -> bool {
        false
    }
}
