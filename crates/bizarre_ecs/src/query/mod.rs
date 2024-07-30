use std::marker::PhantomData;

use query_data::QueryData;
use query_iterator::QueryIterator;

use crate::world::World;

pub mod fetch;
pub mod query_data;
pub mod query_element;
pub mod query_iterator;
pub mod res;

pub struct Query<'q, D: QueryData> {
    world: &'q World,
    _phantom: PhantomData<&'q D>,
}

impl<'q, D: QueryData> Query<'q, D> {
    pub fn new(world: &'q World) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }
}

impl<'q, D> IntoIterator for Query<'q, D>
where
    D: QueryData,
{
    type Item = D::Item<'q>;

    type IntoIter = QueryIterator<'q, D>;

    fn into_iter(self) -> Self::IntoIter {
        let entities = self
            .world
            .components
            .filter_entities(D::inner_type_ids().as_slice());

        QueryIterator::new(self.world, entities)
    }
}
