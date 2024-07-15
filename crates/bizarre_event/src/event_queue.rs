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

    pub fn register_reader<'a, E>(&mut self, reader: EventReader) -> Result<()>
    where
        E: Event,
    {
        if reader.id >= self.next_reader_id {
            return Err(anyhow!("Cannot add a reader not created by this EventQueue (reader id: {}, last created reader: {})", reader.id, self.next_reader_id - 1));
        }

        if let Some(queue) = self.get_queue_mut::<E>() {
            queue.add_reader(reader);
        } else {
            let mut q = TypedEventQueue::new::<E>();
            q.add_reader(reader);
            self.queues.insert(TypeId::of::<E>(), q);
        }

        Ok(())
    }

    pub fn push_event<E>(&mut self, event: E) -> Result<()>
    where
        E: Event,
    {
        if let Some(q) = self.get_queue_mut::<E>() {
            q.push_event(event);
        } else {
            let mut q = TypedEventQueue::new::<E>();
            q.push_event(event);
            self.queues.insert(TypeId::of::<E>(), q);
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

    pub fn pull_events<E>(&mut self, reader: &EventReader) -> Result<Option<Box<[E]>>>
    where
        E: Event + Clone,
    {
        self.get_queue_mut::<E>()
            .ok_or(anyhow!("Cannot pull events: There is no queue to read"))
            .map(|q| q.pull_events(reader))?
    }

    pub fn change_frames(&mut self) {
        self.queues.values_mut().for_each(|q| q.swap_buffers());
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

    use crate::EventQueue;

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
