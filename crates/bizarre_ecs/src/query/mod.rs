use std::marker::PhantomData;

use query_data::QueryData;
use query_iterator::QueryIterator;

use crate::world::World;

pub mod fetch;
pub mod query_data;
pub mod query_element;
pub mod query_iterator;
pub mod res;

pub struct Query<'q, D: QueryData<'q>> {
    world: &'q World,
    _phantom: PhantomData<&'q D>,
}

impl<'q, D: QueryData<'q>> Query<'q, D> {
    pub fn new(world: &'q World) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }
}

impl<'q, D: QueryData<'q>> IntoIterator for Query<'q, D> {
    type Item = D::Item;

    type IntoIter = QueryIterator<'q, D>;

    fn into_iter(self) -> Self::IntoIter {
        let entities = self
            .world
            .components
            .filter_entities(D::inner_type_ids().as_slice());

        QueryIterator::new(self.world, entities)
    }
}
