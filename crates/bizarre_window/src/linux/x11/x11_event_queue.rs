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

    pub fn handle_event(
        &mut self,
        xcb_event: xcb::Event,
    ) -> Result<(), X11WindowEventConvertError> {
        let event = WindowEvent::try_from(xcb_event)?;

        let window_handle = event.window_handle();

        match self.window_events.get_mut(&window_handle) {
            Some(q) => q.push_back(event),
            None => {
                let mut q = VecDeque::default();
                q.push_back(event);
                self.window_events.insert(window_handle, q);
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum X11WindowEventConvertError {
    #[error(
        "The provided event cannot be converted to WindowEvent: it's not a window event ({0:?})"
    )]
    NotAWindowEvent(xcb::Event),
}

impl TryFrom<xcb::Event> for WindowEvent {
    type Error = X11WindowEventConvertError;

    fn try_from(xcb_event: xcb::Event) -> Result<Self, Self::Error> {
        match xcb_event {
            xcb::Event::X(ref ev) => match ev {
                x::Event::ConfigureNotify(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resource_id());
                    let x = ev.x();
                    let y = ev.y();
                    let width = ev.width();
                    let height = ev.height();
                    let size = UVec2::new(width as u32, height as u32);
                    let position = IVec2::new(x as i32, y as i32);
                    Ok(Self::X11ConfigureNotify {
                        handle,
                        size,
                        position,
                    })
                }
                x::Event::DestroyNotify(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resource_id());
                    Ok(Self::WindowClosed(handle))
                }
                x::Event::ClientMessage(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resource_id());
                    Ok(Self::X11ClientMessage {
                        handle,
                        data: ev.data(),
                    })
                }
                _ => Err(Self::Error::NotAWindowEvent(xcb_event)),
            },
            _ => Err(Self::Error::NotAWindowEvent(xcb_event)),
        }
    }
}
