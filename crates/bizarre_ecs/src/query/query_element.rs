use std::any::TypeId;

use crate::{entity::Entity, world::World};

pub trait QueryElement<'q> {
    type Item;

    /// Returns inner type id of the underlying component;
    ///
    /// Must return None if `QueryElement` does not fetch a
    /// [`Component`][crate::component::Component] from [`World`]
    fn inner_type_id() -> Option<TypeId>;
    fn get_item(world: &'q World, entity: Entity) -> Self::Item;
}
