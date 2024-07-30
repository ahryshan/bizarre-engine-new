use std::{any::TypeId, ptr::NonNull};

use crate::{entity::Entity, world::World};

pub trait QueryElement {
    type Item<'a>;

    /// Returns inner type id of the underlying component;
    ///
    /// Must return None if `QueryElement` does not fetch a
    /// [`Component`][crate::component::Component] from [`World`]
    fn inner_type_id() -> Option<TypeId>;
    fn is_non_component() -> bool;
    fn get_item<'a>(world: &'a World, entity: Entity) -> Self::Item<'a>;
}
