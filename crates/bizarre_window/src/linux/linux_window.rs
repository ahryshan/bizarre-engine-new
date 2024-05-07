use std::sync::LazyLock;

use crate::WindowTrait;

use super::{wayland::wayland_window::WaylandWindow, x11::x11_window::X11Window};

pub struct LinuxWindow {
    inner: Box<dyn WindowTrait>,
}

#[derive(Clone, Copy, Debug)]
enum __Display {
    X11,
    Wayland,
}

static DISPLAY: LazyLock<__Display> = LazyLock::new(|| {
    if let Ok(value) = std::env::var("__BE_FORCE_DISPLAY") {
        match value.as_str() {
            "x11" => {
                #[cfg(not(feature = "x11"))]
                panic!("Cannot run with __BE_FORCE_DISPLAY=x11 when X11 support is not included into the compilation");
                __Display::X11
            }
            "wayland" => {
                #[cfg(not(feature = "wayland"))]
                panic!("Cannot run with __BE_FORCE_DISPLAY=wayland when Wayland support is not included into the compilation");
                __Display::Wayland
            }
            _ => {
                panic!("Unknown __BE_FORCE_DISPLAY value: {value}")
            }
        }
    } else if cfg!(all(feature = "wayland", feature = "x11")) {
        match std::env::var("WAYLAND_DISPLAY") {
            Ok(_) => __Display::Wayland,
            Err(_) => __Display::X11,
        }
    } else if cfg!(all(feature = "wayland", not(feature = "x11"))) {
        __Display::Wayland
    } else if cfg!(all(feature = "x11", not(feature = "wayland"))) {
        __Display::X11
    } else {
        panic!("Failed to resolve display server!")
    }
});

impl WindowTrait for LinuxWindow {
    fn new(create_info: &crate::WindowCreateInfo) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let inner: Box<dyn WindowTrait> = match *DISPLAY {
            __Display::X11 => Box::new(X11Window::new(create_info)?),
            __Display::Wayland => Box::new(WaylandWindow::new(create_info)?),
        };

        Ok(Self { inner })
    }

    fn size(&self) -> nalgebra_glm::UVec2 {
        self.inner.size()
    }

    fn position(&self) -> nalgebra_glm::IVec2 {
        self.inner.position()
    }

    fn update_size_and_position(
        &mut self,
    ) -> anyhow::Result<(nalgebra_glm::UVec2, nalgebra_glm::IVec2)> {
        self.inner.update_size_and_position()
    }

    fn mode(&self) -> crate::WindowMode {
        self.inner.mode()
    }

    fn raw_handle(&self) -> u32 {
        self.inner.raw_handle()
    }

    fn title(&self) -> &str {
        self.inner.title()
    }

    fn status(&self) -> crate::window::WindowStatus {
        self.inner.status()
    }

    fn set_size(&mut self, size: nalgebra_glm::UVec2) -> anyhow::Result<()> {
        self.inner.set_size(size)
    }

    fn set_position(&mut self, position: nalgebra_glm::IVec2) -> anyhow::Result<()> {
        self.inner.set_position(position)
    }

    fn set_mode(&mut self, mode: crate::WindowMode) -> anyhow::Result<()> {
        self.inner.set_mode(mode)
    }

    fn set_title(&mut self, title: String) -> anyhow::Result<()> {
        self.inner.set_title(title)
    }

    fn set_decorations(&mut self, decorations: bool) -> anyhow::Result<()> {
        self.inner.set_decorations(decorations)
    }

    fn map(&mut self) -> anyhow::Result<()> {
        self.inner.map()
    }

    fn unmap(&mut self) -> anyhow::Result<()> {
        self.inner.unmap()
    }

    fn minimize(&mut self) -> anyhow::Result<()> {
        self.inner.minimize()
    }

    fn restore(&mut self) -> anyhow::Result<()> {
        self.inner.restore()
    }

    fn maximize(&mut self) -> anyhow::Result<()> {
        self.inner.maximize()
    }

    fn unmaximize(&mut self) -> anyhow::Result<()> {
        self.inner.unmaximize()
    }

    fn close_requested(&self) -> bool {
        self.inner.close_requested()
    }
}
