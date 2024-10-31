use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{
        system::{schedule::Schedule, system_config::IntoSystemConfigs},
        world::{ecs_module::EcsModule, World},
    },
    ecs_modules::{InputModule, WindowModule},
    event::Events,
    input::event::{InputEvent, InputEventSource},
    log::info,
    prelude::*,
    window::{window_manager::WindowManager, WindowCreateInfo},
};

use nalgebra_glm::UVec2;

struct MainEcsModule;

impl EcsModule for MainEcsModule {
    fn apply(self, world: &mut World) {
        world.add_systems(Schedule::Update, listen_pointer_moves);
        world.add_systems(
            Schedule::Update,
            (
                // This works, but there must be specified both `before` and `after` for each systems
                // Trying to add Third, Second, First will fail...
                third_system.after((first_system, second_system)),
                first_system.before((second_system, third_system)),
                second_system.after(first_system).before(third_system),
            ),
        );
    }
}

struct Timer {
    repeat: bool,
    cycle_duration: Duration,
    last_start: Option<Instant>,
}

impl Timer {
    fn new(repeat: bool, cycle_duration: Duration) -> Self {
        Self {
            repeat,
            cycle_duration,
            last_start: None,
        }
    }

    fn tick(&mut self) -> bool {
        let last_start = self.last_start.get_or_insert(Instant::now());

        if last_start.elapsed() >= self.cycle_duration {
            if self.repeat {
                self.last_start = Some(Instant::now());
            }

            true
        } else {
            false
        }
    }
}

fn first_system(mut timer: Local<Option<Timer>>) {
    let timer = timer.get_or_insert(Timer::new(true, Duration::from_secs(1)));

    if timer.tick() {
        info!("First System!");
    }
}

fn second_system(mut timer: Local<Option<Timer>>) {
    let timer = timer.get_or_insert(Timer::new(true, Duration::from_secs(1)));

    if timer.tick() {
        info!("Second System!");
    }
}

fn third_system(mut timer: Local<Option<Timer>>) {
    let timer = timer.get_or_insert(Timer::new(true, Duration::from_secs(1)));

    if timer.tick() {
        info!("Third System!");
    }
}

fn listen_pointer_moves(input_events: Events<InputEvent>) {
    if let Some(events) = input_events.as_ref() {
        for event in events {
            if let InputEvent::ButtonPress { button, .. } = event {
                info!("ButtonPress: {button:?}");
            } else if let InputEvent::ButtonRelease { button, .. } = event {
                info!("ButtonRelease: {button:?}");
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
