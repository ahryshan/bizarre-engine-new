use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs_modules::{InputModule, WindowModule},
    window::WindowCreateInfo,
};

use nalgebra_glm::UVec2;

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(WindowModule::new().with_window(
            WindowCreateInfo::normal_window("Bizarre Window".into(), UVec2::new(800, 600)),
            true,
        ))
        .with_module(InputModule)
        .build()
        .run()
}
