use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use anyhow::{anyhow, Result};

use crate::{event::Event, event_reader::EventReader};

pub struct TypedEventQueue {
    pub(crate) type_id: TypeId,
    front: Vec<Box<dyn Any>>,
    back: Vec<Box<dyn Any>>,
    readers: HashMap<EventReader, usize>,
}

impl TypedEventQueue {
    pub fn new<E>() -> Self
    where
        E: Event + 'static,
    {
        Self {
            type_id: TypeId::of::<E>(),
            front: Default::default(),
            back: Default::default(),
            readers: Default::default(),
        }
    }

    pub fn push_event<E>(&mut self, event: E)
    where
        E: Event + Sized + 'static,
    {
        self.back.push(Box::new(event))
    }

    pub fn poll_event<E>(&mut self, reader: &EventReader) -> Result<Option<&E>>
    where
        E: Event + 'static,
    {
        let reader_index = self.readers.get_mut(reader).ok_or(anyhow!(
            "Reader with id = {} is not registered with this event queue!",
            reader.id
        ))?;

        if *reader_index >= self.front.len() {
            return Ok(None);
        }

        let ev = self.front.iter().skip(*reader_index).enumerate().next();

        match ev {
            Some((index, ev)) => {
                let ev = ev.downcast_ref::<E>().unwrap();
                *reader_index += index;
                Ok(Some(ev))
            }
            None => Ok(None),
        }
    }

    pub fn add_reader(&mut self, reader: EventReader) {
        if let None = self.readers.get(&reader) {
            self.readers.insert(reader, 0);
        }
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.back.clear();
    }
}
