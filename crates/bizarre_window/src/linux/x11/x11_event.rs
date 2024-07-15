use crate::window::WindowHandle;

pub enum X11WindowEvent {
    ConfigureNotify {
        handle: WindowHandle,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    DestroyNotify {
        handle: WindowHandle,
    },
}
