use std::{marker::PhantomData, ptr};

use super::World;

#[derive(Clone, Copy)]
pub struct UnsafeWorldCell<'w>(*mut World, PhantomData<&'w World>);

impl<'w> UnsafeWorldCell<'w> {
    pub unsafe fn new(world: &'w World) -> Self {
        Self(ptr::from_ref(world).cast_mut(), PhantomData)
    }

    pub unsafe fn get(&'w self) -> &'w World {
        self.0.as_ref().unwrap()
    }

    pub unsafe fn get_mut(&'w self) -> &'w mut World {
        self.0.as_mut().unwrap()
    }
}
