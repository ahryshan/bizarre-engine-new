use bizarre_utils::mass_impl;

use crate::{
    component::Component, entity::Entity, resource::ResourceId,
    world::unsafe_world_cell::UnsafeWorldCell,
};

pub trait QueryData {
    type Item<'w>;

    fn resource_ids() -> Vec<ResourceId>;
    unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_>;
}

impl<T> QueryData for &T
where
    T: Component,
{
    type Item<'w> = &'w T;

    fn resource_ids() -> Vec<ResourceId> {
        vec![T::id()]
    }

    unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_> {
        world
            .component(entity)
            .unwrap_or_else(|| panic!("Failed to get {} for {entity:?}", T::name()))
    }
}

macro_rules! impl_query_data {
    ($($el:tt),+) => {
        #[allow(non_snake_case)]
        impl<$($el),+> QueryData for ($($el,)+)
        where
            $($el: QueryData),+
        {
            type Item<'w> = ($($el::Item<'w>,)+);

        fn resource_ids() -> Vec<ResourceId> {
            vec![$($el::resource_ids()),+].into_iter().flatten().collect()
        }

        unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_> {
            ($($el::get_item(world, entity),)+)
        }

        }
    };
}

mass_impl!(impl_query_data, 16, D);
