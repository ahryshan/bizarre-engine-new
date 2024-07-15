use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

use anyhow::{anyhow, Result};

use crate::{event::Event, event_reader::EventReader};

type IteratorType<'frame> = std::slice::Iter<'frame, Box<dyn Any>>;

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
            Ok(None)
        } else {
            let event = self
                .front
                .get(*reader_index)
                .map(|ev| ev.downcast_ref::<E>().unwrap());

            *reader_index += 1;
            Ok(event)
        }
    }

    pub fn pull_events<E>(&mut self, reader: &EventReader) -> Result<Option<Box<[E]>>>
    where
        E: Event + Clone,
    {
        let reader_index = self.readers.get_mut(reader).ok_or(anyhow!(
            "Reader with id = {} is not registered with this event queue!",
            reader.id
        ))?;

        if *reader_index >= self.front.len() {
            return Ok(None);
        }

        let result = self
            .front
            .iter()
            .skip(*reader_index)
            .map(|ev| (&*ev).downcast_ref::<E>().unwrap())
            .cloned()
            .collect::<Box<[E]>>();

        *reader_index += result.len();

        Ok(Some(result))
    }

    pub fn add_reader(&mut self, reader: EventReader) {
        if let None = self.readers.get(&reader) {
            self.readers.insert(reader, 0);
        }
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.back.clear();
        self.readers.iter_mut().for_each(|(_, index)| *index = 0)
    }
}
