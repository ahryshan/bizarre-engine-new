use std::collections::{BTreeMap, VecDeque};

use anyhow::Result;
use nalgebra_glm::{IVec2, UVec2};
use thiserror::Error;
use xcb::{x, Xid};

use crate::{window::WindowHandle, window_events::WindowEvent};

#[derive(Default)]
pub struct X11EventQueue {
    window_events: BTreeMap<WindowHandle, VecDeque<WindowEvent>>,
}

impl X11EventQueue {
    pub fn get_events_for_window(
        &mut self,
        window_handle: WindowHandle,
    ) -> Option<Box<[WindowEvent]>> {
        self.window_events.get_mut(&window_handle).map(|q| {
            if q.len() > 0 {
                let boxed_slice = q.drain(..).collect::<Box<[_]>>();
                Some(boxed_slice)
            } else {
                None
            }
        })?
    }
}
