use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use anyhow::{anyhow, Result};

use crate::{event::Event, event_reader::EventReader};

type IteratorType<'frame> = std::slice::Iter<'frame, Box<dyn Any>>;

pub struct TypedEventQueue {
    pub(crate) type_id: TypeId,
    pub(crate) event_name: &'static str,
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
            event_name: type_name::<E>(),
            front: Default::default(),
            back: Default::default(),
            readers: Default::default(),
        }
    }

    pub fn push_event<E>(&mut self, event: E)
    where
        E: Event + Sized + 'static,
    {
        if TypeId::of::<E>() != self.type_id {
            panic!(
                "Trying to push an `Event` of type `{}` into a queue of type `{}`",
                type_name::<E>(),
                self.event_name,
            )
        }
        self.back.push(Box::new(event))
    }

    pub fn poll_event<E>(&mut self, reader: &EventReader) -> Option<&E>
    where
        E: Event + 'static,
    {
        let reader_index = self
            .readers
            .get_mut(reader)
            .unwrap_or_else(|| panic!("Trying to poll event with an unregistered `EventReader`"));

        if *reader_index >= self.front.len() {
            None
        } else {
            let event = self.front.get(*reader_index).map(|ev| {
                ev.downcast_ref::<E>().unwrap_or_else(|| {
                    panic!(
                        "Trying to poll event `{}` from the queue of type `{}`",
                        type_name::<E>(),
                        self.event_name
                    )
                })
            });

            *reader_index += 1;
            event
        }
    }

    pub fn pull_events<E>(&mut self, reader: &EventReader) -> Vec<E>
    where
        E: Event + Clone,
    {
        let reader_index = self
            .readers
            .get_mut(reader)
            .unwrap_or_else(|| panic!("Trying to poll event with an unregistered `EventReader`"));

        if *reader_index >= self.front.len() {
            return Vec::new();
        }

        let result = self
            .front
            .iter()
            .skip(*reader_index)
            .map(|ev| {
                (*ev).downcast_ref::<E>().unwrap_or_else(|| {
                    panic!(
                        "Trying to pull events of type `{}` from the queue of type `{}`",
                        type_name::<E>(),
                        self.event_name
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>();

        *reader_index += result.len();

        result
    }

    pub fn add_reader(&mut self, reader: EventReader) {
        self.readers.entry(reader).or_insert(0);
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.back.clear();
        self.readers.iter_mut().for_each(|(_, index)| *index = 0)
    }
}
