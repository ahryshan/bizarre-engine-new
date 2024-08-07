use std::collections::HashMap;

use bizarre_ecs::prelude::*;
use bizarre_event::EventQueue;

use crate::{
    linux::x11::connection::{get_x11_context, get_x11_context_mut},
    window_error::{WindowError, WindowResult},
    Window, WindowCreateInfo, WindowHandle, WindowTrait,
};

#[derive(Resource, Default)]
pub struct WindowManager {
    windows: HashMap<WindowHandle, Window>,
    main_window: Option<WindowHandle>,
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
}

#[cfg(target_os = "linux")]
impl WindowManager {
    pub fn drain_window_events(&self, events: &mut EventQueue) -> WindowResult<()> {
        let context = get_x11_context_mut();

        context.drain_system_events(events)?;

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
impl WindowManager {
    pub fn drain_window_events(&self, events: &mut EventQueue) -> WindowResult<()> {
        todo!("Bizarre Engine support windowing only on Linux at the moment");
    }
}
