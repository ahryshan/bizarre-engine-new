use std::any::TypeId;

use crate::{entity::Entity, world::World};

use super::query_element::QueryElement;

pub trait QueryData<'q> {
    type Item;

    fn inner_type_ids() -> Vec<TypeId>;
    fn is_non_component() -> bool;
    fn get_item(world: &'q World, entity: Entity) -> Self::Item;
}

macro_rules! impl_data {
    ($head:tt, $($tail:tt),+) => {
        impl<'q, $head, $($tail),+> QueryData<'q> for ($head, $($tail),+)
        where
            $head: QueryElement<'q>,
            $($tail: QueryElement<'q>),+
        {
            type Item = (<$head as QueryElement<'q>>::Item,
                $(<$tail as QueryElement<'q>>::Item),+);

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id(), $($tail::inner_type_id()),+]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &'q World, entity: Entity) -> Self::Item {
                ($head::get_item(world, entity), $($tail::get_item(world, entity)),+)
            }

            fn is_non_component() -> bool {
                [$head::is_non_component(), $($tail::is_non_component()),+].iter().fold(true, |acc, curr| acc && *curr)
            }
        }

        impl_data!($($tail),+);
    };

    ($head:tt) => {
        impl<'q, $head> QueryData<'q> for $head
        where
            $head: QueryElement<'q>,
        {
            type Item = <$head as QueryElement<'q>>::Item;

            fn inner_type_ids() -> Vec<TypeId> {
                vec![$head::inner_type_id()]
                    .into_iter()
                    .flatten()
                    .collect()
            }

            fn get_item(world: &'q World, entity: Entity) -> Self::Item {
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
