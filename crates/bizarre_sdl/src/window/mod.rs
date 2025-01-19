use std::collections::BTreeMap;

use bizarre_core::Handle;
use bizarre_ecs::prelude::Resource;

use crate::context::with_sdl_video;

mod create_info;

pub use sdl::video::Window;

pub use create_info::WindowCreateInfo;
pub use create_info::WindowPosition;
pub type WindowHandle = Handle<Window>;

#[derive(Default, Resource)]
pub struct Windows {
    windows: BTreeMap<WindowHandle, Window>,
    main_window: Option<WindowHandle>,
}

impl Windows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_window(&mut self, create_info: &WindowCreateInfo) -> WindowHandle {
        let window = with_sdl_video(|video| create_info.builder(video).build()).unwrap();

        let handle = WindowHandle::from_raw(window.id() as usize);

        self.windows.insert(handle, window);

        handle
    }

    pub fn window(&self, handle: &WindowHandle) -> Option<&Window> {
        self.windows.get(handle)
    }

    pub fn window_mut(&mut self, handle: &WindowHandle) -> Option<&mut Window> {
        self.windows.get_mut(handle)
    }

    pub fn remove_window(&mut self, handle: &WindowHandle) -> Option<Window> {
        self.windows.remove(handle)
    }

    pub fn set_main_window(&mut self, handle: WindowHandle) {
        self.main_window = Some(handle)
    }

    pub fn get_main_window(&self) -> Option<&Window> {
        self.windows.get(self.main_window.as_ref()?)
    }
}

pub fn create_test_window() -> Window {
    with_sdl_video(|video| {
        video
            .window("Bizarre engine test window", 0, 0)
            .vulkan()
            .hidden()
            .build()
    })
    .unwrap()
}
