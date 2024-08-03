use std::ops::{Deref, DerefMut};

use bizarre_utils::mass_impl;

use crate::{resource::Resource, world::unsafe_world_cell::UnsafeWorldCell};

pub trait SystemParam {
    type Item<'w, 's>;
    type State;

    unsafe fn init(world: UnsafeWorldCell) -> Self::State;

    unsafe fn get_item<'w, 's>(
        world: UnsafeWorldCell<'w>,
        param_state: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized;
}

pub type SystemParamItem<'w, 's, P> = <P as SystemParam>::Item<'w, 's>;
pub type SystemParamState<P> = <P as SystemParam>::State;

pub struct Res<'w, T>
where
    T: Resource,
{
    value: &'w T,
}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type Item<'w, 's> = Res<'w, T>;

    type State = ();

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {}

    unsafe fn get_item<'w, 's>(
        world: UnsafeWorldCell<'w>,
        _: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        Res {
            value: world.resource().unwrap(),
        }
    }
}

impl<T> Deref for Res<'_, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct ResMut<'w, T>
where
    T: Resource,
{
    value: &'w mut T,
}

impl<T: Resource> SystemParam for ResMut<'_, T> {
    type Item<'w, 's> = ResMut<'w, T>;

    type State = ();

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {}

    unsafe fn get_item<'w, 's>(
        world: UnsafeWorldCell<'w>,
        _: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        ResMut {
            value: world.resource_mut().unwrap(),
        }
    }
}

impl<T: Resource> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: Resource> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct Local<'s, T> {
    value: &'s mut T,
}

impl<T> SystemParam for Local<'_, T>
where
    T: 'static + Default,
{
    type Item<'w, 's> = Local<'s, T>;

    type State = T;

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {
        T::default()
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
}

impl<T> Deref for Local<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for Local<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

macro_rules! impl_system_param {
    ($($param:tt),+) => {
        impl<$($param),+> SystemParam for ($($param,)+)
        where
            $($param: SystemParam),+
        {
            type Item<'w, 's> = ($($param::Item<'w, 's>,)+);
            type State = ($($param::State,)+);

            unsafe fn init(world: UnsafeWorldCell) -> Self::State {
                ($($param::init(world),)+)
            }

            unsafe fn get_item<'w, 's>(world: UnsafeWorldCell<'w>, param_state: &'s mut Self::State) -> Self::Item<'w, 's>
            where
                Self: Sized
            {
                let ($($param,)+) = param_state;
                ($($param::get_item(world, $param),)+)
            }
        }
    };
}

mass_impl!(impl_system_param, 16, F);
