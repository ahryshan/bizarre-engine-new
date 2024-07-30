use std::marker::PhantomData;

use crate::{entity::Entity, world::World};

use super::query_data::QueryData;

pub struct QueryIterator<'q, T>
where
    T: QueryData,
{
    world: &'q World,
    entities: Vec<Entity>,
    index: usize,
    yielded_non_component: bool,
    _phantom: PhantomData<&'q T>,
}

impl<'q, T> QueryIterator<'q, T>
where
    T: QueryData,
{
    pub(crate) fn new(world: &'q World, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            index: 0,
            yielded_non_component: false,
            _phantom: PhantomData,
        }
    }
}

impl<'q, D> Iterator for QueryIterator<'q, D>
where
    D: QueryData,
{
    type Item = D::Item<'q>;

    fn next(&mut self) -> Option<Self::Item> {
        if D::is_non_component() {
            if !self.yielded_non_component {
                return Some(unsafe { D::get_item(self.world.into(), Entity::from_gen_id(0, 0)) });
            } else {
                return None;
            }
        }

        if self.index >= self.entities.len() {
            return None;
        }

        let entity = self.entities[self.index];

        let item = unsafe { D::get_item(self.world.into(), entity) };
        self.index += 1;
        Some(item)
    }
}
