use std::collections::HashMap;

use bizarre_ecs::prelude::*;
use bizarre_event::EventQueue;
use cfg_if::cfg_if;

use crate::{
    window::Window,
    window_error::{WindowError, WindowResult},
    PlatformWindow, WindowCreateInfo, WindowHandle,
};

#[derive(Resource, Default)]
pub struct WindowManager {
    pub(crate) windows: HashMap<WindowHandle, Window>,
    pub(crate) main_window: Option<WindowHandle>,
}

impl WindowManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_window(
        &mut self,
        create_info: &WindowCreateInfo,
    ) -> WindowResult<(WindowHandle, &mut Window)> {
        let window = Window::new(create_info)?;
        let handle = window.handle();
        self.windows.insert(handle, window);

        let ret = self.windows.get_mut(&handle).map(|w| (handle, w)).unwrap();

        Ok(ret)
    }

    pub fn set_main_window(&mut self, handle: WindowHandle) {
        self.main_window = Some(handle)
    }

    pub fn get_main_window_handle(&self) -> Option<WindowHandle> {
        self.main_window
    }

    pub fn get_main_window(&self) -> Option<&Window> {
        self.main_window.as_ref().map(|h| self.windows.get(h))?
    }

    pub fn get_window(&self, handle: &WindowHandle) -> WindowResult<&Window> {
        self.windows.get(handle).ok_or(WindowError::InvalidHandle)
    }

    pub fn iter(&self) -> impl Iterator<Item = (WindowHandle, &Window)> {
        self.windows.iter().map(|(h, w)| (*h, w))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (WindowHandle, &mut Window)> {
        self.windows.iter_mut().map(|(h, w)| (*h, w))
    }
}

#[cfg(target_os = "linux")]
use crate::linux::linux_window::{__LinuxDisplay, get_linux_display_type};

#[cfg(all(target_os = "linux", feature = "x11"))]
use crate::linux::x11::connection::*;

#[cfg(target_os = "linux")]
impl WindowManager {
    pub fn drain_window_events(&self, events: &mut EventQueue) -> WindowResult<()> {
        match get_linux_display_type() {
            __LinuxDisplay::X11 => self.drain_window_events_x11(events),
            __LinuxDisplay::Wayland => self.drain_window_events_wl(events),
        }
    }

    fn drain_window_events_x11(&self, events: &mut EventQueue) -> WindowResult<()> {
        cfg_if! {
            if #[cfg(feature = "x11")] {
                let context = get_x11_context_mut();

                context.drain_system_events(events)
            } else {
                panic!("Trying to pump events from X11 server while there is no X11 support included into compilation");
            }
        }
    }

    fn drain_window_events_wl(&self, events: &mut EventQueue) -> WindowResult<()> {
        cfg_if! {
            if #[cfg(feature = "wayland")] {
                use crate::linux::wayland::wl_context::WL_CONTEXT;

                let mut context = WL_CONTEXT.write().unwrap();

                context.drain_system_events(events)
            } else {
                panic!("Trying to pump events from Wayland server while there is no Wayland support included into compilation");
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
impl WindowManager {
    pub fn drain_window_events(&self, events: &mut EventQueue) -> WindowResult<()> {
        todo!("Bizarre Engine support windowing only on Linux at the moment");
    }
}
