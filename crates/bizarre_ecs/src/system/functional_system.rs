use std::{any::type_name, marker::PhantomData};

use bizarre_utils::mass_impl;

use crate::world::unsafe_world_cell::UnsafeWorldCell;

use super::{
    system_param::{SystemParamItem, SystemParamState},
    IntoSystem, System, SystemParam,
};

pub trait FnSys<Marker> {
    type Param: SystemParam;

    fn run(&mut self, param_value: SystemParamItem<Self::Param>);
}

pub struct FunctionalSystem<Marker, F>
where
    F: FnSys<Marker>,
{
    func: F,
    init: bool,
    param_state: Option<SystemParamState<F::Param>>,
    _phantom: PhantomData<Marker>,
}

impl<Marker, F> System for FunctionalSystem<Marker, F>
where
    F: FnSys<Marker>,
{
    fn is_init(&self) -> bool {
        self.init
    }

    fn init(&mut self, world: UnsafeWorldCell) {
        self.param_state = Some(unsafe { F::Param::init(world) });
        self.init = true;
    }

    fn run(&mut self, world: UnsafeWorldCell) {
        let param_value = unsafe { F::Param::get_item(world, self.param_state.as_mut().unwrap()) };
        self.func.run(param_value)
    }

    fn name_static() -> &'static str {
        type_name::<F>()
    }

    fn name(&self) -> &'static str {
        Self::name_static()
    }
}

impl<Marker, F> IntoSystem<Marker> for F
where
    F: FnSys<Marker> + 'static,
    Marker: 'static,
{
    type System = FunctionalSystem<Marker, F>;

    fn into_system(self) -> Self::System {
        Self::System {
            func: self,
            init: false,
            param_state: None,
            _phantom: PhantomData,
        }
    }
}

macro_rules! impl_fn_sys {
    ($($param:tt),+) => {
        #[allow(non_snake_case)]
        impl<$($param,)+ F> FnSys<fn($($param),+)> for F
        where
            for<'a> &'a mut F: FnMut($($param),+) + FnMut($(SystemParamItem<$param>),+),
            $($param: SystemParam),+
        {
            type Param = ($($param,)+);

            fn run(&mut self, param_value: SystemParamItem<Self::Param>) {
                #[allow(non_snake_case)]
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($param),+>(mut f: impl FnMut($($param),+), $($param: $param),+) {
                    f($($param),+)
                }

                let ($($param,)+) = param_value;
                call_inner(self, $($param),+)
            }
        }
    };
}

mass_impl!(impl_fn_sys, 16, F);
