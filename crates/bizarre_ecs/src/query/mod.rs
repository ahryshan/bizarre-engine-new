use std::marker::PhantomData;

use query_element::QueryData;

use crate::{
    entity::Entity,
    system::{system_param::SystemParam, WorldAccess},
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

pub mod query_element;

#[derive(Clone)]
pub struct Query<'q, D: QueryData> {
    world: UnsafeWorldCell<'q>,
    _phantom: PhantomData<D>,
}

impl<'q, D: QueryData> Query<'q, D> {
    pub fn new(world: &'q World) -> Self {
        let world = unsafe { world.as_unsafe_cell() };

        Self {
            world,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> QueryIterator<'q, D> {
        QueryIterator {
            world: self.world,
            entities: self.world.filter_entities(D::resource_ids().as_slice()),
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'q, D: QueryData> SystemParam for Query<'q, D> {
    type Item<'w, 's> = Query<'w, D>;

    type State = ();

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {}

    unsafe fn get_item<'w, 's>(
        world: UnsafeWorldCell<'w>,
        _: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        Query {
            world,
            _phantom: PhantomData,
        }
    }

    fn param_access() -> Vec<WorldAccess> {
        D::query_access()
    }
}

impl<'q, D: QueryData> IntoIterator for Query<'q, D> {
    type Item = D::Item<'q>;

    type IntoIter = QueryIterator<'q, D>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIterator {
            world: self.world,
            entities: self.world.filter_entities(D::resource_ids().as_slice()),
            index: 0,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct QueryIterator<'q, D: QueryData> {
    world: UnsafeWorldCell<'q>,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<D>,
}

impl<'a, D: QueryData> Iterator for QueryIterator<'a, D> {
    type Item = D::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entity = *self.entities.get(self.index)?;
        self.index += 1;
        Some(unsafe { D::get_item(self.world, entity) })
    }
}
