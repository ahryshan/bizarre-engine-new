use bizarre_utils::mass_impl;

use crate::{
    component::Component,
    entity::Entity,
    resource::ResourceId,
    system::{WorldAccess, WorldAccessType},
    world::unsafe_world_cell::UnsafeWorldCell,
};

pub trait QueryData {
    type Item<'w>;

    fn resource_ids() -> Vec<ResourceId>;
    unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_>;
    fn query_access() -> Vec<WorldAccess>;
}

impl<T> QueryData for &T
where
    T: Component,
{
    type Item<'w> = &'w T;

    fn resource_ids() -> Vec<ResourceId> {
        vec![T::resource_id()]
    }

    unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_> {
        world
            .component(entity)
            .unwrap_or_else(|| panic!("Failed to get {} for {entity:?}", T::resource_name()))
    }

    fn query_access() -> Vec<WorldAccess> {
        vec![WorldAccess {
            resource_id: T::resource_id(),
            resource_name: T::resource_name(),
            access_type: WorldAccessType::CompRead,
        }]
    }
}

impl<T> QueryData for &mut T
where
    T: Component,
{
    type Item<'w> = &'w mut T;

    fn resource_ids() -> Vec<ResourceId> {
        vec![T::resource_id()]
    }

    unsafe fn get_item(world: UnsafeWorldCell, entity: Entity) -> Self::Item<'_> {
        world
            .component_mut(entity)
            .unwrap_or_else(|| panic!("Failed to get {} for {entity:?}", T::resource_name()))
    }

    fn query_access() -> Vec<WorldAccess> {
        vec![WorldAccess {
            resource_id: T::resource_id(),
            resource_name: T::resource_name(),
            access_type: WorldAccessType::CompWrite,
        }]
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

            fn query_access() -> Vec<WorldAccess> {
                let mut access = vec![];
                $(access.extend($el::query_access());)+
                access
            }

        }

    };
}

mass_impl!(impl_query_data, 16, D);
