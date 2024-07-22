use std::any::TypeId;

use crate::world::World;

use super::{
    component_storage::Component, query_element::QueryElement, query_iterator::QueryIterator,
};

pub trait QueryData<'q> {
    type Item;

    fn inner_type_ids() -> Vec<TypeId>;
    fn iter<I>(iters: Vec<I>) -> QueryIterator<'q, Self, impl Iterator<Item = Self::Item>>
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

    fn iter<I>(iters: Vec<I>) -> QueryIterator<'q, Self, impl Iterator<Item = Self>>
    where
        Self: Sized,
        I: Iterator<Item = Component> + Clone,
    {
        let [ref iter] = iters[0..1] else {
            panic!("Trying to build a QueryIterator from data with insufficient number of members");
        };

        let iter = iter.clone().map(T::from_component);

        QueryIterator {
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

    fn iter<I>(iters: Vec<I>) -> QueryIterator<'q, Self, impl Iterator<Item = Self::Item>>
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
            iter,
            _phantom: Default::default(),
        }
    }
}
