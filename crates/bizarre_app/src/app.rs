use std::time::{Duration, Instant};

use anyhow::Result;
use bizarre_event::{EventQueue, EventReader};

use crate::app_event::AppEvent;

pub struct App {
    name: String,
    event_queue: EventQueue,
    event_reader: EventReader,
    running: bool,
    paused: bool,
}

impl App {
    pub fn new(name: String) -> Self {
        let mut event_queue = EventQueue::default();
        let event_reader = event_queue.create_reader();
        event_queue
            .register_reader::<AppEvent>(event_reader)
            .unwrap();

        Self {
            name,
            event_queue,
            event_reader,
            running: false,
            paused: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.running = true;

        const FRAME_TARGET_TIME: Duration = Duration::from_millis(1000 / 60);

        let mut frame = 0;

        while self.running {
            let frame_start = Instant::now();

            println!("Frame #{frame}");
            frame += 1;

            self.event_queue.change_frames();

            self.pump_app_events()?;

            let frame_end = Instant::now();
            let frame_duration = frame_end - frame_start;

            if frame_duration <= FRAME_TARGET_TIME {
                std::thread::sleep(FRAME_TARGET_TIME - frame_duration);
            }
        }

        Ok(())
    }

    fn pump_app_events(&mut self) -> Result<()> {
        while let Some(ev) = self
            .event_queue
            .poll_event::<AppEvent>(&self.event_reader)?
        {
            match ev {
                AppEvent::CloseRequested => {
                    println!("Got AppEvent::CloseRequested!");
                    self.running = false;
                    self.event_queue.push_event(AppEvent::WillClose)?
                }
                _ => {}
            }
        }

        Ok(())
    }
}
