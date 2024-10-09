use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{
        system::schedule::Schedule,
        world::{ecs_module::EcsModule, World},
    },
    ecs_modules::{InputModule, WindowModule},
    event::Events,
    input::event::{InputEvent, InputEventSource},
    log::info,
    prelude::Res,
    window::{window_manager::WindowManager, WindowCreateInfo},
};

use nalgebra_glm::UVec2;

struct MainEcsModule;

impl EcsModule for MainEcsModule {
    fn apply(self, world: &mut World) {
        world.add_systems(Schedule::Update, listen_pointer_moves)
    }
}

fn listen_pointer_moves(window_manager: Res<WindowManager>, input_events: Events<InputEvent>) {
    if let Some(events) = input_events.as_ref() {
        for event in events {
            if let InputEvent::PointerMove {
                source,
                position,
                delta,
            } = event
            {
                let [x, y] = (*position).into();
                let [dx, dy] = (*delta).into();
                let InputEventSource::Window(handle) = source;
                let window = window_manager.get_window(handle).unwrap();
                info!(
                    "{}: pointer moved to ({x:.2}, {y:.2}) (d: {dx:.2}, {dy:.2})",
                    window.title()
                )
            }
        }
    }
}

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(
            WindowModule::new().with_window(
                WindowCreateInfo::normal_window("Bizarre Window".into(), UVec2::new(800, 600)),
                true,
            ), // .with_window(
               //     WindowCreateInfo::normal_window(
               //         "Bizarre Window 2".into(),
               //         UVec2::new(600, 800),
               //     ),
               //     false,
               // ),
        )
        .with_module(InputModule)
        .with_module(MainEcsModule)
        .build()
        .run()
}
