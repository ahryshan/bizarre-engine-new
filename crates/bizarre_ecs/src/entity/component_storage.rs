use std::{
    any::Any,
    cell::RefCell,
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use thiserror::Error;

use super::EntityId;

pub type Component = Rc<RefCell<dyn Any>>;

pub struct ComponentStorage {
    inner: Vec<Option<Component>>,
}

impl From<Vec<Option<Component>>> for ComponentStorage {
    fn from(value: Vec<Option<Component>>) -> Self {
        Self { inner: value }
    }
}

impl Deref for ComponentStorage {
    type Target = Vec<Option<Component>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ComponentStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Index<T> for ComponentStorage
where
    T: Into<EntityId>,
{
    type Output = Option<Component>;

    fn index(&self, index: T) -> &Self::Output {
        let index: usize = index.into().into();
        &self.inner[index]
    }
}

impl<T> IndexMut<T> for ComponentStorage
where
    T: Into<EntityId>,
{
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let index: usize = index.into().into();
        &mut self.inner[index]
    }
}
