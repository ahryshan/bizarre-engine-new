use std::marker::PhantomData;

use bizarre_utils::mass_impl;

use crate::{
    commands::command_buffer::CommandBuffer,
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

use super::{
    system_param::{SystemParamItem, SystemParamState},
    IntoSystem, System, SystemParam, WorldAccess, WorldAccessType,
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

    fn apply_deferred(&mut self, world: &mut World) {
        if let Some(mut cmd) = self.take_deferred() {
            cmd.apply(world)
        }
    }

    fn take_deferred(&mut self) -> Option<CommandBuffer> {
        F::Param::take_deferred(self.param_state.as_mut().unwrap())
    }

    fn access() -> Box<[WorldAccess]> {
        F::Param::param_access().into()
    }
}

impl<Marker, F> IntoSystem<Marker> for F
where
    F: FnSys<Marker> + 'static,
    Marker: 'static,
{
    type System = FunctionalSystem<Marker, F>;

    fn into_system(self) -> Self::System {
        // #[cfg(debug_assertions)]
        // {
        //     if let Some(conflicts) = get_internal_conflicts(&meta.access) {
        //         let msg = conflicts.into_iter().enumerate().fold(
        //             format!(
        //                 "Failed to build system `{}`, found access conflicts:\n",
        //                 meta.name
        //             ),
        //             |acc, (num, msg)| format!("{acc}\t{}. {}\n", num + 1, msg),
        //         );
        //         panic!("{msg}");
        //     }
        // }

        Self::System {
            func: self,
            init: false,
            param_state: None,
            _phantom: PhantomData,
        }
    }
}

#[cfg(debug_assertions)]
fn get_internal_conflicts(access: &[WorldAccess]) -> Option<Vec<String>> {
    let internal_conflicts = access
        .chunk_by(|a, b| {
            a.resource_id == b.resource_id
                && a.access_type & WorldAccessType::ResourceMask
                    == b.access_type & WorldAccessType::ResourceMask
        })
        .filter_map(|chunk| {
            if chunk.len() < 2 {
                return None;
            }
            let reads = chunk
                .iter()
                .filter(|a| a.access_type.intersects(WorldAccessType::Read))
                .cloned()
                .collect::<Vec<_>>();
            let writes = chunk
                .iter()
                .filter(|a| a.access_type.intersects(WorldAccessType::Write))
                .cloned()
                .collect::<Vec<_>>();

            if writes.len() > 1 {
                Some(format!("multiple {}", writes[0]))
            } else if !writes.is_empty() && !reads.is_empty() {
                Some(format!("{} while accessing it immutably", writes[0]))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !internal_conflicts.is_empty() {
        Some(internal_conflicts)
    } else {
        None
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

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        query::Query,
        system::system_param::{Res, ResMut},
    };

    #[derive(Resource)]
    struct Res1;

    #[derive(Component)]
    struct Comp1;

    fn multiple_mutable_res(_: ResMut<Res1>, _: ResMut<Res1>) {}

    fn multiple_mutable_query(_: Query<(&mut Comp1, &mut Comp1)>) {}

    fn mut_and_ref_res(_: Res<Res1>, _: ResMut<Res1>) {}

    fn mut_and_ref_query(_: Query<(&mut Comp1, &Comp1)>) {}

    #[test]
    #[should_panic]
    fn should_panic_on_multiple_mutable_access() {
        multiple_mutable_res.into_system();
    }

    #[test]
    #[should_panic]
    fn should_panic_on_multiple_mutable_query_access() {
        multiple_mutable_query.into_system();
    }

    #[test]
    #[should_panic]
    fn should_panic_on_mut_and_ref_resource_access() {
        mut_and_ref_res.into_system();
    }

    #[test]
    #[should_panic]
    fn should_panic_on_mut_and_ref_query_access() {
        mut_and_ref_query.into_system();
    }
}
