use std::sync::Once;

use anyhow::Result;
use bizarre_event::EventQueue;
use nalgebra_glm::{IVec2, UVec2};
use thiserror::Error;
use xcb::{
    x::{self, ChangeKeyboardControl},
    xkb, Xid,
};

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
        CONTEXT = xcb::Connection::connect_with_extensions(None, &[xcb::Extension::Xkb], &[])
            .map_err(|err| panic!("Cannot connect to X11 display: {:?}", err))
            .map(|(connection, screen_num)| X11Context {
                conn: connection,
                screen_num,
            })
            .unwrap()
            .into();

        let X11Context { conn, .. } = CONTEXT.as_mut().unwrap();

        // let repeat_cookie = conn.send_request(&xkb::PerClientFlags {
        //     device_spec: xkb::Id::UseCoreKbd as u16,
        //     change: xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
        //     value: xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
        //     ctrls_to_change: xkb::BoolCtrl::REPEAT_KEYS,
        //     auto_ctrls: xkb::BoolCtrl::REPEAT_KEYS,
        //     auto_ctrls_values: xkb::BoolCtrl::REPEAT_KEYS,
        // });

        // let reply = conn.wait_for_reply(repeat_cookie);

        let cookie = conn.send_request_checked(&ChangeKeyboardControl {
            value_list: &[x::Kb::AutoRepeatMode(x::AutoRepeatMode::Off)],
        });

        conn.check_request(cookie)
            .map_err(|err| panic!("{err:?}"))
            .unwrap();
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
            match X11WindowEvent::try_from(xcb_event) {
                Ok(ev) => event_queue.push_event(ev),
                Err(X11WindowEventConvertError::UnsupportedX11Event(ev)) => {
                    println!("Unhandled X11 event: {ev:?}")
                }
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum X11WindowEventConvertError {
    #[error(
        "The provided event cannot be converted to WindowEvent: it's not a supported X11 event ({0:?})"
    )]
    UnsupportedX11Event(xcb::Event),
}

impl TryFrom<xcb::Event> for X11WindowEvent {
    type Error = X11WindowEventConvertError;

    fn try_from(xcb_event: xcb::Event) -> Result<Self, Self::Error> {
        match xcb_event {
            xcb::Event::X(ref ev) => match ev {
                x::Event::ConfigureNotify(ev) => {
                    let handle = WindowHandle::from_raw(ev.window().resouce_id());
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
                x::Event::KeyPress(ev) => {
                    let handle = WindowHandle::from_raw(ev.event().resource_id());
                    let keycode = ev.detail();
                    Ok(Self::KeyPress { handle, keycode })
                }
                x::Event::KeyRelease(ev) => {
                    let handle = WindowHandle::from_raw(ev.event().resource_id());
                    let keycode = ev.detail();
                    Ok(Self::KeyRelease { handle, keycode })
                }
                x::Event::ButtonPress(ev) => {
                    let handle = WindowHandle::from_raw(ev.event().resource_id());
                    let pos = IVec2::new(ev.event_x() as i32, ev.event_y() as i32);
                    let keycode = ev.detail();
                    Ok(Self::ButtonPress {
                        handle,
                        pos,
                        keycode,
                    })
                }
                x::Event::ButtonRelease(ev) => {
                    let handle = WindowHandle::from_raw(ev.event().resource_id());
                    let pos = IVec2::new(ev.event_x() as i32, ev.event_y() as i32);
                    let keycode = ev.detail();
                    Ok(Self::ButtonRelease {
                        handle,
                        pos,
                        keycode,
                    })
                }
                x::Event::MotionNotify(ev) => {
                    let handle = WindowHandle::from_raw(ev.event().resource_id());
                    let pos = IVec2::new(ev.event_x() as i32, ev.event_y() as i32);
                    Ok(Self::MouseMove { handle, pos })
                }
                _ => Err(Self::Error::UnsupportedX11Event(xcb_event)),
            },
            _ => Err(Self::Error::UnsupportedX11Event(xcb_event)),
        }
    }
}
