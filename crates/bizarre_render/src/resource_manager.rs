use std::{
    collections::{hash_map::Entry, HashMap},
    sync::atomic::{AtomicUsize, Ordering},
};

use bizarre_core::Handle;

use crate::present_target::{PresentTarget, PresentTargetHandle};

pub struct RenderResourceManager<T> {
    pub(crate) resources: HashMap<Handle<T>, T>,
    pub(crate) next_id: AtomicUsize,
}

impl<T> Default for RenderResourceManager<T> {
    fn default() -> Self {
        Self {
            resources: Default::default(),
            next_id: AtomicUsize::new(1),
        }
    }
}

impl<T> RenderResourceManager<T> {
    pub fn insert(&mut self, resource: T) -> Handle<T> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let handle = Handle::<T>::from_raw(id);
        self.resources.insert(handle, resource);
        handle
    }

    pub fn remove(&mut self, handle: &Handle<T>) -> Option<T> {
        self.resources.remove(handle)
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.resources.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.resources.get_mut(handle)
    }
}
