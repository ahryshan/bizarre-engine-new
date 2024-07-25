use std::marker::PhantomData;

use crate::{entity::Entity, resource::Resource, world::World};

use super::query_element::QueryElement;

pub struct Res<'q, T>(PhantomData<&'q T>)
where
    T: Resource;

impl<'q, T> QueryElement<'q> for Res<'q, T>
where
    T: Resource,
{
    type Item = &'q T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        None
    }

    fn get_item(world: &'q World, _: Entity) -> Self::Item {
        world.resources.get::<T>().unwrap()
    }
}

pub struct ResMut<'q, T>(PhantomData<&'q T>)
where
    T: Resource;

impl<'q, T> QueryElement<'q> for ResMut<'q, T>
where
    T: Resource,
{
    type Item = &'q mut T;

    fn inner_type_id() -> Option<std::any::TypeId> {
        None
    }

    fn get_item(world: &'q World, _: Entity) -> Self::Item {
        world.resources.get_mut::<T>().unwrap()
    }
}
