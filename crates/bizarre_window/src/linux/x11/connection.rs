use std::{
    collections::BTreeMap,
    sync::{LazyLock, Once},
};

use xcb::{x, Event};

use crate::window::WindowHandle;

#[derive(Default)]
pub struct X11EventQueue {
    window_events: BTreeMap<WindowHandle, xcb::Event>,
    common_events: Vec<xcb::Event>,
}

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
