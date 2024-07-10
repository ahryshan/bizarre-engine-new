use std::{any::TypeId, collections::HashMap};

use anyhow::{anyhow, Result};

use crate::{event::Event, event_reader::EventReader, typed_event_queue::TypedEventQueue};

pub struct EventQueue {
    queues: HashMap<TypeId, TypedEventQueue>,
    next_reader_id: usize,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self {
            next_reader_id: 1,
            queues: Default::default(),
        }
    }
}

impl EventQueue {
    pub fn create_reader(&mut self) -> EventReader {
        let id = self.next_reader_id;
        self.next_reader_id += 1;
        EventReader { id }
    }

    pub fn register_reader<E>(&mut self, reader: EventReader) -> Result<()>
    where
        E: Event,
    {
        if reader.id >= self.next_reader_id {
            return Err(anyhow!("Cannot add a reader not created by this EventQueue (reader id: {}, last created reader: {})", reader.id, self.next_reader_id - 1));
        }

        match self.get_queue_mut::<E>() {
            Some(queue) => queue.add_reader(reader),
            None => {
                self.add_queue::<E>()
                    .map_err(|err| anyhow!("Cannot add reader: {err}"))?;
                self.get_queue_mut::<E>().unwrap().add_reader(reader);
            }
        }

        Ok(())
    }

    pub fn push_event<E>(&mut self, event: E) -> Result<()>
    where
        E: Event,
    {
        match self.get_queue_mut::<E>() {
            Some(q) => q.push_event(event),
            None => {
                self.add_queue::<E>()
                    .map_err(|err| anyhow!("Failed to push event to queue: {err}"))?;
                self.get_queue_mut::<E>().unwrap().push_event(event);
            }
        }
        Ok(())
    }

    pub fn poll_event<E>(&mut self, reader: &EventReader) -> Result<Option<&E>>
    where
        E: Event,
    {
        self.get_queue_mut::<E>()
            .ok_or(anyhow!("Cannot poll events: There is no queue to read"))
            .map(|q| q.poll_event(reader))?
    }

    pub fn change_frames(&mut self) {
        self.queues.values_mut().for_each(|q| q.swap_buffers());
    }

    fn add_queue<E>(&mut self) -> Result<()>
    where
        E: Event,
    {
        let ev_type_id = TypeId::of::<E>();
        if self.queues.contains_key(&ev_type_id) {
            return Err(anyhow!(
                "Cannot add sub-queue for a event {}. There is already one",
                std::any::type_name::<E>()
            ));
        }

        self.queues.insert(ev_type_id, TypedEventQueue::new::<E>());
        Ok(())
    }

    #[inline(always)]
    fn get_queue<E>(&self) -> Option<&TypedEventQueue>
    where
        E: Event,
    {
        let ev_type_id = TypeId::of::<E>();
        self.queues.get(&ev_type_id)
    }

    #[inline(always)]
    fn get_queue_mut<E>(&mut self) -> Option<&mut TypedEventQueue>
    where
        E: Event,
    {
        let ev_type_id = TypeId::of::<E>();
        self.queues.get_mut(&ev_type_id)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{Event, EventQueue};

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct TestEvent1 {
        str_info: &'static str,
        usize_info: usize,
    }

    #[test]
    fn event_reader_should_be_created_with_adequate_id() {
        let mut event_queue = EventQueue::default();
        let reader = event_queue.create_reader();
        assert!(reader.id == 1)
    }

    #[test]
    fn event_reader_should_register() -> Result<()> {
        let mut event_queue = EventQueue::default();
        let reader = event_queue.create_reader();
        event_queue.register_reader::<TestEvent1>(reader)?;
        Ok(())
    }

    #[test]
    fn event_should_be_polled() -> Result<()> {
        let mut event_queue = EventQueue::default();
        let reader = event_queue.create_reader();
        event_queue.register_reader::<TestEvent1>(reader)?;

        let event = TestEvent1 {
            str_info: "Hello world!",
            usize_info: 0,
        };

        event_queue.push_event(event)?;
        event_queue.change_frames();
        let polled_event = event_queue.poll_event::<TestEvent1>(&reader)?;

        assert!(
            polled_event == Some(&event),
            "Expected {:?}, found {polled_event:?}",
            Some(&event)
        );

        Ok(())
    }
}
