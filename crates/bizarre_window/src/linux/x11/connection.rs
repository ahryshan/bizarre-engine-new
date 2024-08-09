use std::sync::Once;

use anyhow::Result;
use bizarre_event::EventQueue;
use nalgebra_glm::{IVec2, UVec2};
use thiserror::Error;
use xcb::{x, Xid};

use crate::{
    window_error::{WindowError, WindowResult},
    window_events::WindowEvent,
    WindowHandle,
};

use super::{x11_event::X11WindowEvent, x11_event_queue::X11EventQueue};

pub struct X11Context {
    pub conn: xcb::Connection,
    pub screen_num: i32,
}

static mut CONTEXT: Option<X11Context> = None;
static CONTEXT_INIT: Once = Once::new();

fn init_x11_context() {
    unsafe {
        CONTEXT = xcb::Connection::connect(None)
            .map_err(|err| panic!("Cannot connect to X11 display: {:?}", err))
            .map(|(connection, screen_num)| X11Context {
                conn: connection,
                screen_num,
            })
            .unwrap()
            .into();
    }
}

pub fn get_x11_context() -> &'static X11Context {
    CONTEXT_INIT.call_once(init_x11_context);

    unsafe { CONTEXT.as_ref().unwrap() }
}

pub fn get_x11_context_mut() -> &'static mut X11Context {
    CONTEXT_INIT.call_once(init_x11_context);

    unsafe { CONTEXT.as_mut().unwrap() }
}

impl X11Context {
    pub fn drain_system_events(&mut self, event_queue: &mut EventQueue) -> WindowResult<()> {
        while let Some(xcb_event) = self.conn.poll_for_event().map_err(WindowError::from)? {
            if let Ok(ev) = X11WindowEvent::try_from(xcb_event) {
                event_queue.push_event(ev)
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

impl TryFrom<xcb::Event> for X11WindowEvent {
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
                    Ok(Self::ConfigureNotify {
                        handle,
                        size,
                        position,
                    })
                }
                x::Event::DestroyNotify(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resource_id());
                    Ok(Self::DestroyNotify { handle })
                }
                x::Event::ClientMessage(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resource_id());
                    Ok(Self::ClientMessage {
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
