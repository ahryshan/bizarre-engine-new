use std::any::TypeId;

use crate::{entity::Entity, world::World};

use super::query_element::QueryElement;

pub trait QueryData<'q> {
    type Item;

    fn inner_type_ids() -> Vec<TypeId>;
    fn get_item(world: &'q World, entity: Entity) -> Self::Item;
}

macro_rules! impl_data {
    ($head:tt, $($tail:tt),+) => {
        impl<'q, $head, $($tail),+> QueryData<'q> for ($head, $($tail),+)
        where
            $head: QueryElement<'q>,
            $($tail: QueryElement<'q>),+
        {
            type Item = ($head::Item, $($tail::Item),+);

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id(), $($tail::inner_type_id()),+]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &'q World, entity: Entity) -> Self::Item {
                ($head::get_item(world, entity), $($tail::get_item(world, entity)),+)
            }
        }

        impl_data!($($tail),+);
    };

    ($head:tt) => {
        impl<'q, $head> QueryData<'q> for $head
        where
            $head: QueryElement<'q>,
        {
            type Item = $head::Item;

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id()]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &'q World, entity: Entity) -> Self::Item {
                $head::get_item(world, entity)
            }
        }
    };

    () => {}
}

impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
