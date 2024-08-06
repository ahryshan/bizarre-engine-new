use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use bizarre_utils::mass_impl;

use crate::{
    commands::command_buffer::CommandBuffer,
    resource::Resource,
    system::WorldAccessType,
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

use super::WorldAccess;

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

    fn param_access() -> Vec<WorldAccess>;

    fn take_deferred(state: &mut Self::State) -> Option<CommandBuffer> {
        let _ = state;
        None
    }
}

impl SystemParam for () {
    type Item<'w, 's> = ();

    type State = ();

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {}

    unsafe fn get_item<'w, 's>(_: UnsafeWorldCell<'w>, _: &'s mut Self::State) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
    }

    fn param_access() -> Vec<WorldAccess> {
        vec![]
    }
}

pub type SystemParamItem<'w, 's, P> = <P as SystemParam>::Item<'w, 's>;
pub type SystemParamState<P> = <P as SystemParam>::State;

pub struct Res<'w, T>
where
    T: Resource,
{
    value: &'w T,
}

impl<T: Debug + Resource> Debug for Res<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Res").field("value", &self.value).finish()
    }
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
            value: world
                .resource()
                .unwrap_or_else(|| panic!("Failed to get resource `{}`", T::resource_name())),
        }
    }

    fn param_access() -> Vec<WorldAccess> {
        vec![WorldAccess {
            resource_id: T::resource_id(),
            resource_name: T::resource_name(),
            access_type: WorldAccessType::ResRead,
        }]
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

    fn param_access() -> Vec<WorldAccess> {
        vec![WorldAccess {
            access_type: WorldAccessType::ResWrite,
            resource_name: T::resource_name(),
            resource_id: T::resource_id(),
        }]
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

pub trait FromWorld {
    fn from_world(world: &mut World) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_: &mut World) -> Self {
        Self::default()
    }
}

pub struct Local<'s, T> {
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
        #[allow(non_snake_case)]
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

            fn param_access() -> Vec<WorldAccess> {
                let mut access = vec![];
                $(access.extend($param::param_access());)+
                access
            }

            fn take_deferred(state: &mut Self::State) -> Option<CommandBuffer> {
                let ($($param,)+) = state;
                let cmd = vec![$($param::take_deferred($param)),+]
                    .into_iter()
                    .flatten()
                    .fold(CommandBuffer::new(), |mut acc, mut curr| {
                        acc.append(&mut curr);
                        acc
                    });
                if !cmd.is_empty() {
                    Some(cmd)
                } else {
                    None
                }
            }
        }
    };
}

mass_impl!(impl_system_param, 16, F);
