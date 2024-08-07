use anyhow::Result;
use bizarre_engine::app::app_state::{AppRunTime, DeltaTime};
use bizarre_engine::app::{App, AppBuilder};

use bizarre_engine::ecs::system::schedule::Schedule;
use bizarre_engine::ecs::world::ecs_module::EcsModule;
use bizarre_engine::prelude::*;

struct MainEcsModule;

impl EcsModule for MainEcsModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        world.add_systems(Schedule::Update, print_timers);
    }
}

fn print_timers(mut counter: Local<usize>, delta: Res<DeltaTime>, run_time: Res<AppRunTime>) {
    println!(
        "#{counter}: delta - {}ms, run_time: {}",
        delta.as_millis(),
        *run_time
    );
    *counter += 1;
}

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(MainEcsModule)
        .build()
        .run()
}
