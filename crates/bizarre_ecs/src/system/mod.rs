use system_param::SystemParam;

use crate::world::unsafe_world_cell::UnsafeWorldCell;

pub mod functional_system;
pub mod system_graph;
pub mod system_param;

pub trait System {
    fn run(&mut self, world: UnsafeWorldCell);

    fn init(&mut self, world: UnsafeWorldCell);

    fn is_init(&self) -> bool;

    fn name_static() -> &'static str
    where
        Self: Sized;

    fn name(&self) -> &'static str;
}

pub trait IntoSystem<Marker> {
    type System: System + 'static;

    fn into_system(self) -> Self::System;
}

#[cfg(test)]
mod tests {
    use crate::{
        system::{
            system_graph::SystemGraph,
            system_param::{Local, Res, ResMut},
            IntoSystem,
        },
        world::World,
    };

    struct DeltaTime(pub f64);

    #[test]
    fn types_should_work() {
        fn system(delta: Res<DeltaTime>, delta_mut: ResMut<DeltaTime>) {
            let _ = delta;
            let _ = delta_mut;
        }

        fn use_system<Marker>(system: impl IntoSystem<Marker>) {
            let _ = system;
        }

        use_system(system)
    }

    #[test]
    fn should_run_system() {
        fn delta_time_system(delta: Res<DeltaTime>, mut time_counter: Local<f64>) {
            *time_counter += delta.0;
            println!("Frame time: {}s, runtime: {}s", delta.0, *time_counter);
        }

        let mut world = World::new();

        world.insert_resource(DeltaTime(0.16));

        let mut sg = SystemGraph::new();

        sg.add_system(delta_time_system);

        sg.iter().for_each(|s| println!("System: {}", s.name()));

        sg.init_systems(&world);
        sg.run_systems(&mut world);
        sg.run_systems(&mut world);
        sg.run_systems(&mut world);
        sg.run_systems(&mut world);
        sg.run_systems(&mut world);
    }
}
