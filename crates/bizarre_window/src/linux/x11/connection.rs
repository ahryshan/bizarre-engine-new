use std::sync::Once;

use anyhow::Result;

use super::x11_event_queue::X11EventQueue;

pub struct X11Context {
    pub conn: xcb::Connection,
    pub screen_num: i32,
    pub event_queue: X11EventQueue,
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
                event_queue: Default::default(),
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
    pub fn drain_system_events(&mut self) -> Result<()> {
        while let Some(xcb_event) = self.conn.poll_for_event()? {
            self.event_queue.handle_event(xcb_event);
        }

        Ok(())
    }
}
