use std::ops::{Deref, DerefMut};

use anyhow::Result;
use bizarre_core::Handle;
use bizarre_event::{EventQueue, EventReader};
use cfg_if::cfg_if;
use nalgebra_glm::{IVec2, UVec2};

use crate::{
    platform_window::PlatformWindow, window_error::WindowResult, window_events::WindowEvent,
    WindowCreateInfo,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum WindowMode {
    Fullscreen,
    #[default]
    Windowed,
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

impl Window {
    pub fn new(create_info: &WindowCreateInfo) -> WindowResult<Self> {
        let inner = {
            cfg_if! {
                if #[cfg(target_os = "linux")] {
                    use crate::linux::linux_window::LinuxWindow;

                    Box::new(LinuxWindow::new(create_info)?)
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
