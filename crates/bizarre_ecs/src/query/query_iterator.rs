use std::marker::PhantomData;

use crate::{entity::Entity, world::World};

use super::query_data::QueryData;

pub struct QueryIterator<'q, T>
where
    T: QueryData<'q>,
{
    world: &'q World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<&'q T>,
}

impl<'q, T> QueryIterator<'q, T>
where
    T: QueryData<'q>,
{
    pub(crate) fn new(world: &'q World, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'q, D> Iterator for QueryIterator<'q, D>
where
    D: QueryData<'q>,
{
    type Item = D::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entities.len() {
            return None;
        }

        let entity = self.entities[self.index];

        let item = D::get_item(self.world, entity);
        self.index += 1;
        Some(item)
    }
}
