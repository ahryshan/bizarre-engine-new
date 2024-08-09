use anyhow::Result;
use bizarre_engine::app::app_state::{AppRunTime, DeltaTime};
use bizarre_engine::app::{App, AppBuilder};

use bizarre_engine::ecs::system::schedule::Schedule;
use bizarre_engine::ecs::world::ecs_module::EcsModule;
use bizarre_engine::ecs_modules::WindowModule;
use bizarre_engine::prelude::*;
use bizarre_engine::window::{PlatformWindow, WindowCreateInfo};
use nalgebra_glm::UVec2;

struct MainEcsModule;

impl EcsModule for MainEcsModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {}
}

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(WindowModule::new().with_window(
            WindowCreateInfo::normal_window("Bizarre Window".into(), UVec2::new(800, 600)),
            true,
        ))
        .with_module(MainEcsModule)
        .build()
        .run()
}
