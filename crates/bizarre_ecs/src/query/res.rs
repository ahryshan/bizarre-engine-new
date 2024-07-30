use std::{marker::PhantomData, ptr::NonNull};

use crate::{entity::Entity, resource::Resource, world::World};

use super::query_element::QueryElement;

pub struct Res<T>(PhantomData<T>)
where
    T: Resource;

impl<T> QueryElement for Res<T>
where
    T: Resource,
{
    type Item<'a> = &'a T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        None
    }

    fn get_item<'a>(world: &'a World, _: Entity) -> Self::Item<'a> {
        world.resources.get::<T>().unwrap()
    }

    fn is_non_component() -> bool {
        true
    }
}

pub struct ResMut<T>(PhantomData<T>)
where
    T: Resource;

impl<T> QueryElement for ResMut<T>
where
    T: Resource,
{
    type Item<'a> = &'a mut T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        None
    }

    fn get_item<'a>(world: &'a World, _: Entity) -> Self::Item<'a> {
        world.resources.get_mut::<T>().unwrap()
    }

    fn is_non_component() -> bool {
        true
    }
}
