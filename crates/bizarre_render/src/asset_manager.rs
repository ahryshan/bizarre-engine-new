use std::collections::HashMap;

use bizarre_core::{
    handle::{HandlePlacement, HandleStrategy},
    Handle,
};
use bizarre_ecs::prelude::Resource;

#[derive(Resource)]
pub struct AssetStore<A: 'static, S: 'static> {
    data: HashMap<Handle<A>, A>,
    handle_strategy: S,
}

impl<A, S: Default> Default for AssetStore<A, S> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            handle_strategy: Default::default(),
        }
    }
}

impl<A, S: HandleStrategy<A> + Default> AssetStore<A, S> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<A, S: HandleStrategy<A>> AssetStore<A, S> {
    pub fn with_strategy(handle_strategy: S) -> Self {
        Self {
            handle_strategy,
            data: Default::default(),
        }
    }

    pub fn insert(&mut self, asset: A) -> Handle<A> {
        let handle = self.handle_strategy.new_handle(&asset);
        self.data.insert(handle, asset);
        handle
    }

    pub fn contains(&self, handle: Handle<A>) -> HandlePlacement {
        self.handle_strategy.handle_placement(&handle)
    }

    pub fn delete(&mut self, handle: Handle<A>) -> Option<A> {
        self.handle_strategy.mark_deleted(handle);
        self.data.remove(&handle)
    }

    pub fn get(&self, handle: &Handle<A>) -> Option<&A> {
        self.data.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        self.data.get_mut(handle)
    }
}
