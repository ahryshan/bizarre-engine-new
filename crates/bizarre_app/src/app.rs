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

        while self.running {
            self.event_queue.change_frames();

            self.pump_app_events()?;
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
                    self.running = false;
                    self.event_queue.push_event(AppEvent::WillClose)?
                }
                _ => {}
            }
        }

        Ok(())
    }
}
