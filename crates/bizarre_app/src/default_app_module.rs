use std::ops::Deref;
use std::time::{Duration, Instant};

use bizarre_ecs::prelude::*;

use bizarre_ecs::system::schedule::Schedule;
use bizarre_ecs::world::ecs_module::EcsModule;

use crate::app_state::{AppRunTime, DeltaTime};

pub struct DefaultAppEcsModule;

impl EcsModule for DefaultAppEcsModule {
    fn apply(self, world: &mut bizarre_ecs::world::World) {
        world.insert_resource(DeltaTime(Duration::default()));
        world.insert_resource(AppRunTime(Duration::default()));

        world.add_systems(Schedule::Preupdate, update_timers);
    }
}

fn update_timers(
    mut last_frame: Local<Instant>,
    app_start: Local<Instant>,
    mut delta: ResMut<DeltaTime>,
    mut run_time: ResMut<AppRunTime>,
) {
    delta.0 = last_frame.elapsed();
    *last_frame = Instant::now();
    run_time.0 = app_start.elapsed();
}
