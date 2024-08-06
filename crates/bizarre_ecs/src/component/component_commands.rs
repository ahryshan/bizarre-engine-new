use std::marker::PhantomData;

use crate::commands::Command;

use super::component_batch::ComponentBatch;

pub struct RegisterComponentsCmd<T: ComponentBatch> {
    _phantom: PhantomData<T>,
}

impl<T: ComponentBatch> RegisterComponentsCmd<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: ComponentBatch> Command for RegisterComponentsCmd<T> {
    fn apply(self, world: &mut crate::world::World) {
        world.register_components::<T>()
    }
}
