use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use bizarre_core::Handle;
use bizarre_event::EventReader;
use cfg_if::cfg_if;

use crate::{platform_window::PlatformWindow, window_error::WindowResult, WindowCreateInfo};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum WindowMode {
    Fullscreen,
    #[default]
    Windowed,
    WindowedBorderless,
}

pub struct WindowStatus {
    pub minimized: bool,
    pub maximized: bool,
    pub mapped: bool,
}

pub type WindowHandle = Handle<Window>;

pub struct Window {
    handle: WindowHandle,
    inner: Box<dyn PlatformWindow>,
    event_reader: Option<EventReader>,
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("handle", &self.handle)
            .field("event_reader", &self.event_reader)
            .finish_non_exhaustive()
    }
}

impl Window {
    pub fn new(create_info: &WindowCreateInfo) -> WindowResult<Self> {
        let inner = {
            cfg_if! {
                if #[cfg(target_os = "linux")] {
                    use crate::linux::linux_window::create_linux_window;

                    create_linux_window(create_info)?
                } else {
                    todo!("Only linux is supported at the moment")
                }
            }
        };

        Ok(Self {
            handle: inner.handle(),
            inner,
            event_reader: None,
        })
    }

    pub fn handle(&self) -> WindowHandle {
        self.handle
    }
}

impl Deref for Window {
    type Target = dyn PlatformWindow;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}
