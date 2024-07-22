use std::{any::TypeId, marker::PhantomData};

use crate::world::World;

use super::{component_storage::Component, query_element::QueryElement};

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

        D::iter(self.world, storages)
    }
}

pub trait QueryData<'q> {
    type Item;

    fn inner_type_ids() -> Vec<TypeId>;
    fn iter<I>(
        world: &'q World,
        iters: Vec<I>,
    ) -> QueryIterator<'q, Self, impl Iterator<Item = Self::Item>>
    where
        Self: Sized,
        I: Iterator<Item = Component> + Clone;
}

impl<'q, T> QueryData<'q> for T
where
    T: QueryElement,
{
    type Item = T;

    fn inner_type_ids() -> Vec<TypeId> {
        vec![T::inner_type_id()]
    }

    fn iter<I>(
        world: &'q World,
        iters: Vec<I>,
    ) -> QueryIterator<'q, Self, impl Iterator<Item = Self>>
    where
        Self: Sized,
        I: Iterator<Item = Component> + Clone,
    {
        let [ref iter] = iters[0..1] else {
            panic!("Trying to build a QueryIterator from data with insufficient number of members");
        };

        let iter = iter.clone().map(T::from_component);

        QueryIterator {
            world,
            iter,
            _phantom: Default::default(),
        }
    }
}

impl<'q, A, B> QueryData<'q> for (A, B)
where
    A: QueryElement,
    B: QueryElement,
{
    type Item = (A, B);

    fn inner_type_ids() -> Vec<TypeId> {
        vec![A::inner_type_id(), B::inner_type_id()]
    }

    fn iter<I>(
        world: &'q World,
        iters: Vec<I>,
    ) -> QueryIterator<'q, Self, impl Iterator<Item = Self::Item>>
    where
        I: Iterator<Item = Component> + Clone,
    {
        let [ref iter_a, ref iter_b, ..] = iters[..] else {
            panic!("Trying to build a QueryIterator from data with insufficient number of members");
        };

        let mut iter_a = iter_a.clone();
        let mut iter_b = iter_b.clone();

        let count = iter_a.clone().count();

        let iter = (0..count).filter_map(move |_| {
            let items = (iter_a.next(), iter_b.next());
            if let (Some(item_a), Some(item_b)) = items {
                Some((A::from_component(item_a), B::from_component(item_b)))
            } else {
                None
            }
        });

        QueryIterator::<Self, _> {
            world,
            iter,
            _phantom: Default::default(),
        }
    }
}

pub struct QueryIterator<'q, D: QueryData<'q>, I> {
    world: &'q World,
    iter: I,
    _phantom: PhantomData<&'q D>,
}

impl<'q, D: QueryData<'q>, I> Iterator for QueryIterator<'q, D, I>
where
    I: Iterator<Item = D::Item>,
{
    type Item = D::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
