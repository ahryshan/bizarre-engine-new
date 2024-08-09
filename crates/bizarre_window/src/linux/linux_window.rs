use std::sync::LazyLock;

use bizarre_event::EventQueue;
use nalgebra_glm::UVec2;

use crate::{
    platform_window::PlatformWindow,
    window::{WindowHandle, WindowStatus},
    window_error::WindowResult,
    WindowMode,
};

#[cfg(feature = "wayland")]
use super::wayland::wayland_window::WaylandWindow;

#[cfg(feature = "x11")]
use super::x11::x11_window::X11Window;

pub struct LinuxWindow {
    inner: Box<dyn PlatformWindow>,
}

#[derive(Clone, Copy, Debug)]
pub enum __LinuxDisplay {
    X11,
    Wayland,
}

static DISPLAY: LazyLock<__LinuxDisplay> = LazyLock::new(|| {
    if let Ok(value) = std::env::var("__BE_FORCE_DISPLAY") {
        match value.as_str() {
            "x11" => {
                #[cfg(not(feature = "x11"))]
                panic!("Cannot run with __BE_FORCE_DISPLAY=x11 when X11 support is not included into the compilation");
                __LinuxDisplay::X11
            }
            "wayland" => {
                #[cfg(not(feature = "wayland"))]
                panic!("Cannot run with __BE_FORCE_DISPLAY=wayland when Wayland support is not included into the compilation");
                __LinuxDisplay::Wayland
            }
            _ => {
                panic!("Unknown __BE_FORCE_DISPLAY value: {value}")
            }
        }
    } else if cfg!(all(feature = "wayland", feature = "x11")) {
        match std::env::var("WAYLAND_DISPLAY") {
            Ok(_) => __LinuxDisplay::Wayland,
            Err(_) => __LinuxDisplay::X11,
        }
    } else if cfg!(all(feature = "wayland", not(feature = "x11"))) {
        __LinuxDisplay::Wayland
    } else if cfg!(all(feature = "x11", not(feature = "wayland"))) {
        __LinuxDisplay::X11
    } else {
        panic!("Failed to resolve display server!")
    }
});

pub fn get_linux_display_type() -> __LinuxDisplay {
    DISPLAY.clone()
}

impl PlatformWindow for LinuxWindow {
    fn new(create_info: &crate::WindowCreateInfo) -> WindowResult<Self>
    where
        Self: Sized,
    {
        let display = DISPLAY.clone();
        let inner: Box<dyn PlatformWindow> = match display {
            __LinuxDisplay::X11 => Box::new(X11Window::new(create_info)?),
            #[cfg(feature = "wayland")]
            __LinuxDisplay::Wayland => Box::new(WaylandWindow::new(create_info)?),
            _ => panic!(
                "Cannot create window, because support for display server not present: {display:?}"
            ),
        };

        Ok(Self { inner })
    }

    fn size(&self) -> UVec2 {
        self.inner.size()
    }

    fn position(&self) -> nalgebra_glm::IVec2 {
        self.inner.position()
    }

    fn update_size_and_position(&mut self) -> WindowResult<(UVec2, nalgebra_glm::IVec2)> {
        self.inner.update_size_and_position()
    }

    fn mode(&self) -> WindowMode {
        self.inner.mode()
    }

    fn raw_handle(&self) -> u32 {
        self.inner.raw_handle()
    }

    fn handle(&self) -> WindowHandle {
        self.inner.handle()
    }

    fn title(&self) -> &str {
        self.inner.title()
    }

    fn status(&self) -> WindowStatus {
        self.inner.status()
    }

    fn set_size(&mut self, size: UVec2) -> WindowResult<()> {
        self.inner.set_size(size)
    }

    fn set_position(&mut self, position: nalgebra_glm::IVec2) -> WindowResult<()> {
        self.inner.set_position(position)
    }

    fn set_mode(&mut self, mode: WindowMode) -> WindowResult<()> {
        self.inner.set_mode(mode)
    }

    fn set_title(&mut self, title: String) -> WindowResult<()> {
        self.inner.set_title(title)
    }

    fn set_decorations(&mut self, decorations: bool) -> WindowResult<()> {
        self.inner.set_decorations(decorations)
    }

    fn map(&mut self) -> WindowResult<()> {
        self.inner.map()
    }

    fn unmap(&mut self) -> WindowResult<()> {
        self.inner.unmap()
    }

    fn minimize(&mut self) -> WindowResult<()> {
        self.inner.minimize()
    }

    fn restore(&mut self) -> WindowResult<()> {
        self.inner.restore()
    }

    fn maximize(&mut self) -> WindowResult<()> {
        self.inner.maximize()
    }

    fn unmaximize(&mut self) -> WindowResult<()> {
        self.inner.unmaximize()
    }

    fn close_requested(&self) -> bool {
        self.inner.close_requested()
    }

    fn handle_events(&mut self, event_queue: &mut EventQueue) -> WindowResult<()> {
        self.inner.handle_events(event_queue)
    }
}
