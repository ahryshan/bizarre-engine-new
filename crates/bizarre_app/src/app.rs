use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant},
};

use anyhow::Result;
use bizarre_ecs::{
    system::{schedule::Schedule, system_param::ResMut},
    world::World,
};
use bizarre_event::{EventQueue, EventReader};
use bizarre_log::{core_info, info, shutdown_logging};

use crate::app_event::AppEvent;

pub struct App {
    pub(crate) name: String,
    pub(crate) running: bool,
    pub(crate) paused: bool,
    pub(crate) world: World,
    pub(crate) event_reader: EventReader,

    #[cfg(target_os = "linux")]
    pub(crate) termination_receiver: Receiver<i32>,
}

impl App {
    pub fn run(&mut self) -> Result<()> {
        core_info!("Starting the `{}` App", self.name);

        self.running = true;

        const FRAME_TARGET_TIME: Duration = Duration::from_millis(1000 / 60);

        while self.running {
            let frame_start = Instant::now();

            self.world.init_schedule(Schedule::Preupdate);
            self.world.run_schedule(Schedule::Preupdate);

            self.process_app_events();
            self.world.init_schedule(Schedule::Update);
            self.world.run_schedule(Schedule::Update);

            let frame_end = Instant::now();
            let frame_duration = frame_end - frame_start;

            if frame_duration <= FRAME_TARGET_TIME {
                std::thread::sleep(FRAME_TARGET_TIME - frame_duration);
            }
        }

        self.world.purge();
        shutdown_logging();

        Ok(())
    }

    fn process_app_events(&mut self) {
        let event_queue = self.world.resource_mut::<EventQueue>().unwrap();

        while let Some(ev) = event_queue.poll_event::<AppEvent>(&self.event_reader) {
            if let AppEvent::CloseRequested = ev {
                core_info!("Got AppEvent::CloseRequested!");
                self.running = false;
                event_queue.push_event(AppEvent::WillClose);
            }
        }

        #[cfg(target_os = "linux")]
        {
            match self.termination_receiver.try_recv() {
                Ok(_) => event_queue.push_event(AppEvent::CloseRequested),
                Err(TryRecvError::Disconnected) => {
                    panic!("App termination receiver is disconnected!")
                }
                _ => {}
            }
        }
    }
}
