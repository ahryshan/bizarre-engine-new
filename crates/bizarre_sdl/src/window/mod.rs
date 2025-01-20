use std::collections::BTreeMap;

use bizarre_core::Handle;
use bizarre_ecs::prelude::Resource;
use nalgebra_glm::IVec2;
use nalgebra_glm::UVec2;

use crate::context::with_sdl_video;

pub mod create_info;
pub mod window_event;

pub use sdl::video::Window;

pub use create_info::WindowCreateInfo;
pub use create_info::WindowPosition;
pub use window_event::WindowEvent;

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

pub fn try_handle_sdl_event(windows: &Windows, event: &sdl::event::Event) -> Option<WindowEvent> {
    match event {
        sdl::event::Event::Window {
            window_id,
            win_event,
            ..
        } => {
            let handle = WindowHandle::from_raw(*window_id);
            match win_event {
                sdl::event::WindowEvent::Shown => Some(WindowEvent::Shown(handle)),
                sdl::event::WindowEvent::Hidden => Some(WindowEvent::Hidden(handle)),
                sdl::event::WindowEvent::Exposed => Some(WindowEvent::Exposed(handle)),
                sdl::event::WindowEvent::Moved(x, y) => Some(WindowEvent::Moved {
                    handle,
                    pos: IVec2::new(*x, *y),
                }),
                sdl::event::WindowEvent::SizeChanged(w, h) => Some(WindowEvent::Resized {
                    handle,
                    size: UVec2::new(*w as u32, *h as u32),
                }),
                sdl::event::WindowEvent::Minimized => Some(WindowEvent::Minimized(handle)),
                sdl::event::WindowEvent::Maximized => Some(WindowEvent::Maximized(handle)),
                sdl::event::WindowEvent::Restored => Some(WindowEvent::Restored(handle)),
                sdl::event::WindowEvent::Enter => Some(WindowEvent::MouseEnter(handle)),
                sdl::event::WindowEvent::Leave => Some(WindowEvent::MouseLeave(handle)),
                sdl::event::WindowEvent::FocusGained => {
                    Some(WindowEvent::KeyboardFocusGained(handle))
                }
                sdl::event::WindowEvent::FocusLost => Some(WindowEvent::KeyboardFocusLost(handle)),
                sdl::event::WindowEvent::Close => {
                    if Some(handle) == windows.main_window {
                        Some(WindowEvent::MainWindowCloseRequested(handle))
                    } else {
                        Some(WindowEvent::CloseRequested(handle))
                    }
                }
                _ => None,
            }
        }
        _ => None,
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
