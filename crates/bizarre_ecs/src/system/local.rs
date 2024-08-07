use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::world::{unsafe_world_cell::UnsafeWorldCell, World};

use super::{system_param::SystemParam, WorldAccess};

pub struct Local<'s, T: FromWorld> {
    value: &'s mut T,
}

impl<T> SystemParam for Local<'_, T>
where
    T: 'static + FromWorld,
{
    type Item<'w, 's> = Local<'s, T>;

    type State = T;

    unsafe fn init(world: UnsafeWorldCell) -> Self::State {
        T::from_world(world.unsafe_world_mut())
    }

    unsafe fn get_item<'w, 's>(
        _: UnsafeWorldCell<'w>,
        state: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        Local { value: state }
    }

    fn param_access() -> Vec<WorldAccess> {
        vec![]
    }
}

impl<T> Debug for Local<'_, T>
where
    T: FromWorld + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> Display for Local<'_, T>
where
    T: FromWorld + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: FromWorld> Deref for Local<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: FromWorld> DerefMut for Local<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

impl FromWorld for Instant {
    fn from_world(_: &mut World) -> Self {
        Instant::now()
    }
}

macro_rules! impl_generic_default_from_world {
    ($($type:tt),+) => {
        $(
            impl<T> FromWorld for $type<T> {
                fn from_world(_: &mut World) -> Self {
                    Self::default()
                }
            }
        )+
    };
}

macro_rules! impl_default_from_world {
    ($($type:tt),+) => {
        $(
            impl FromWorld for $type {
                fn from_world(_: &mut World) -> Self {
                    Self::default()
                }
            }
        )+
    };
}

impl_default_from_world!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, bool);

impl_default_from_world!(String);

impl_generic_default_from_world!(Vec, Option);
