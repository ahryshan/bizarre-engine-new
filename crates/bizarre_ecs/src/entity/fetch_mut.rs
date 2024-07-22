use std::{cell::RefMut, marker::PhantomData};

use super::{component_storage::Component, query_element::QueryElement};

pub struct FetchMut<T> {
    component: Component,
    _phantom: PhantomData<T>,
}

impl<T> QueryElement for FetchMut<T>
where
    T: 'static,
{
    type Item = T;
    type LockType<'b> = RefMut<'b, T> where Self: 'b;

    fn get_lock(&self) -> Self::LockType<'_> {
        let r = RefMut::map(self.component.borrow_mut(), |r| r.downcast_mut().unwrap());
        r
    }

    fn from_component(component: Component) -> Self {
        Self {
            component,
            _phantom: Default::default(),
        }
    }
}
