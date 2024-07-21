use std::{cell::Ref, marker::PhantomData};

use super::{
    component_storage::Component,
    query::{QueryElement, QueryElementIterator},
};

/// To use in queries on high level
pub struct Fetch<'a, T>
where
    T: 'static,
{
    component: &'a Component,
    _phantom: PhantomData<T>,
}

impl<'a, T> Clone for Fetch<'a, T> {
    fn clone(&self) -> Self {
        Self {
            component: self.component,
            _phantom: self._phantom,
        }
    }
}

impl<'a, T> QueryElement<'a, T> for Fetch<'a, T> {
    type LockType = Ref<'a, T>;
    type Item = T;
    type QEIterator = FetchIterator<'a, T>;

    fn new(component: &'a Component) -> Self {
        Self {
            component,
            _phantom: Default::default(),
        }
    }

    fn get_lock(&self) -> Self::LockType {
        let r = self.component.borrow();
        let r = Ref::map(r, |r| r.downcast_ref().unwrap());
        r
    }

    fn transform_iter<I>(iter: I) -> Self::QEIterator
    where
        I: Iterator<Item = &'a Component> + Clone,
    {
        FetchIterator::from_iter(iter.clone())
    }
}

pub struct FetchIterator<'a, T>
where
    T: 'static,
{
    index: usize,
    iter: Vec<Fetch<'a, T>>,
}

impl<'a, T> QueryElementIterator<'a, Fetch<'a, T>, T> for FetchIterator<'a, T>
where
    T: 'static,
{
    fn from_iter(iter: impl Iterator<Item = &'a Component>) -> Self {
        Self {
            index: 0,
            iter: iter.map(|c| Fetch::new(c)).collect(),
        }
    }
}

impl<'a, T> Iterator for FetchIterator<'a, T>
where
    T: 'static,
{
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.get(self.index).cloned();
        self.index += 1;
        item.map(|i| i.get_lock())
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::entity::entities::Entities;

    #[derive(Debug)]
    struct Health(u32);

    #[test]
    fn should_query_one_component() {
        let mut entities = Entities::new();
        entities.register_component::<Health>();

        entities.spawn().with_component(Health(100)).build();
        entities.spawn().with_component(Health(200)).build();
        entities.spawn().with_component(Health(300)).build();

        let storage = entities.components.get(&TypeId::of::<Health>()).unwrap();
        let storage = storage.iter().filter_map(|c| match c {
            Some(c) => Some(c),
            None => None,
        });

        let iter: FetchIterator<Health> = FetchIterator::from_iter(storage);
        for health in iter {
            eprintln!("{health:?}");
        }

        panic!()
    }
}
