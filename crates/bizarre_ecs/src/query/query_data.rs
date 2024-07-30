use std::any::TypeId;

use crate::{entity::Entity, world::World};

use super::query_element::QueryElement;

pub trait QueryData {
    type Item<'a>;

    fn inner_type_ids() -> Vec<TypeId>;
    fn is_non_component() -> bool;
    fn get_item(world: &World, entity: Entity) -> Self::Item<'_>;
}

impl QueryData for () {
    type Item<'a> = ();

    fn inner_type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn is_non_component() -> bool {
        true
    }

    fn get_item(_: &World, _: Entity) -> Self::Item<'_> {}
}

macro_rules! impl_data {
    ($head:tt, $($tail:tt),+) => {
        impl<$head, $($tail),+> QueryData for ($head, $($tail),+)
        where
            $head: QueryElement,
            $($tail: QueryElement),+
        {
            type Item<'a> = (<$head as QueryElement>::Item<'a>,
                $(<$tail as QueryElement>::Item<'a>),+);

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id(), $($tail::inner_type_id()),+]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &World, entity: Entity) -> Self::Item<'_> {
                ($head::get_item(world, entity), $($tail::get_item(world, entity)),+)
            }

            fn is_non_component() -> bool {
                [$head::is_non_component(), $($tail::is_non_component()),+].iter().fold(true, |acc, curr| acc && *curr)
            }
        }

        impl_data!($($tail),+);
    };

    ($head:tt) => {
        impl<$head> QueryData for $head
        where
            $head: QueryElement,
        {
            type Item<'a> = <$head as QueryElement>::Item<'a>;

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id()]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &World, entity: Entity) -> Self::Item<'_> {
                $head::get_item(world, entity)
            }

            fn is_non_component() -> bool {
                $head::is_non_component()
            }
        }
    };

    () => {}
}

impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
