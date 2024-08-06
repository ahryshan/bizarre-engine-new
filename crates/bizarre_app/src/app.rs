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

use crate::app_event::AppEvent;

pub struct App {
    name: String,
    running: bool,
    paused: bool,
    world: World,
    event_reader: EventReader,

    #[cfg(target_os = "linux")]
    termination_receiver: Receiver<i32>,
}

impl App {
    pub fn new(name: String) -> Self {
        let mut world = World::new();

        let mut event_queue = EventQueue::new();
        let event_reader = event_queue.create_reader();
        event_queue.register_reader::<AppEvent>(event_reader);

        world.insert_resource(event_queue);

        world.add_schedule(Schedule::Init);
        world.add_schedule(Schedule::Update);

        world.add_systems(Schedule::Update, change_event_queue_frames);

        #[cfg(target_os = "linux")]
        let termination_receiver = Self::setup_termination_handler();

        Self {
            name,
            running: false,
            paused: false,
            world,
            event_reader,

            #[cfg(target_os = "linux")]
            termination_receiver,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.running = true;

        const FRAME_TARGET_TIME: Duration = Duration::from_millis(1000 / 60);

        let mut frame = 0;

        self.world.init_schedule(Schedule::Init);
        self.world.run_schedule(Schedule::Init);

        while self.running {
            let frame_start = Instant::now();

            frame += 1;

            self.process_app_events();
            self.world.init_schedule(Schedule::Update);
            self.world.run_schedule(Schedule::Update);

            let frame_end = Instant::now();
            let frame_duration = frame_end - frame_start;

            if frame_duration <= FRAME_TARGET_TIME {
                std::thread::sleep(FRAME_TARGET_TIME - frame_duration);
            }
        }

        Ok(())
    }

    fn process_app_events(&mut self) {
        let event_queue = self.world.resource_mut::<EventQueue>().unwrap();

        while let Some(ev) = event_queue.poll_event::<AppEvent>(&self.event_reader) {
            if let AppEvent::CloseRequested = ev {
                println!("Got AppEvent::CloseRequested!");
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

    #[cfg(target_os = "linux")]
    fn setup_termination_handler() -> Receiver<i32> {
        let (tx, rx) = mpsc::channel();

        ctrlc::set_handler(move || {
            println!("GOT SIGTERM");
            tx.send(0)
                .unwrap_or_else(|_| panic!("Could not send termination signal"));
        });

        rx
    }
}

pub fn change_event_queue_frames(mut eq: ResMut<EventQueue>) {
    eq.change_frames();
}
