use std::{cell::Ref, marker::PhantomData};

use super::{component_storage::Component, query_element::QueryElement};

pub struct Fetch<T> {
    component: Component,
    _phantom: PhantomData<T>,
}

impl<T> QueryElement for Fetch<T>
where
    T: 'static,
{
    type Item = T;
    type LockType<'a> = Ref<'a, T>;

    fn get_lock(&self) -> Self::LockType<'_> {
        let r = Ref::map(self.component.borrow(), |r| r.downcast_ref().unwrap());
        r
    }

    fn from_component(component: Component) -> Self {
        Self {
            component,
            _phantom: Default::default(),
        }
    }
}
