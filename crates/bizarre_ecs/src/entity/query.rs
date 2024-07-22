use std::marker::PhantomData;

use crate::world::World;

use super::{query_data::QueryData, query_iterator::QueryIterator};

pub struct Query<'q, D: QueryData<'q>> {
    world: &'q World,
    _phantom: PhantomData<D>,
}

impl<'q, D: QueryData<'q>> Query<'q, D> {
    pub fn new(world: &'q World) -> Self {
        Self {
            world,
            _phantom: Default::default(),
        }
    }
}

impl<'q, D: QueryData<'q> + 'q> IntoIterator for Query<'q, D> {
    type Item = D::Item;

    type IntoIter = QueryIterator<'q, D, impl Iterator<Item = D::Item> + 'q>;

    fn into_iter(self) -> Self::IntoIter {
        let bitmap = D::inner_type_ids().iter().fold(0, |acc, ti| {
            acc | self.world.entities.comp_bitmasks.get(ti).unwrap()
        });

        let e_ids = self
            .world
            .entities
            .entity_bitmasks
            .iter()
            .enumerate()
            .filter_map(|(e_id, bm)| {
                if *bm & bitmap == bitmap {
                    Some(e_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let storages = D::inner_type_ids()
            .into_iter()
            .map(move |tid| {
                let storage = self.world.entities.components.get(&tid).unwrap();
                let e_ids = e_ids.clone();
                storage.iter().enumerate().filter_map(move |(index, c)| {
                    if !e_ids.contains(&index) {
                        return None;
                    }

                    Some(c.as_ref().unwrap().clone())
                })
            })
            .collect();

        D::iter(storages)
    }
}
