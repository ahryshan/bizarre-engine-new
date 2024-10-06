use std::sync::LazyLock;

use bizarre_event::EventQueue;
use cfg_if::cfg_if;
use nalgebra_glm::UVec2;

use crate::{
    platform_window::PlatformWindow,
    window::{WindowHandle, WindowStatus},
    window_error::WindowResult,
    WindowCreateInfo, WindowMode,
};

#[cfg(feature = "wayland")]
use super::wayland::wl_window::WlWindow;

#[derive(Clone, Copy, Debug)]
pub enum __LinuxDisplay {
    X11,
    Wayland,
}

static DISPLAY: LazyLock<__LinuxDisplay> = LazyLock::new(|| {
    if let Ok(value) = std::env::var("__BE_FORCE_LINUX_DISPLAY") {
        match value.as_str() {
            "x11" => {
                #[cfg(not(feature = "x11"))]
                panic!("Cannot run with __BE_FORCE_LINUX_DISPLAY=x11 when X11 support is not included into the compilation");
                __LinuxDisplay::X11
            }
            "wayland" => {
                #[cfg(not(feature = "wayland"))]
                panic!("Cannot run with __BE_FORCE_LINUX_DISPLAY=wayland when Wayland support is not included into the compilation");
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

pub fn create_linux_window(
    create_info: &WindowCreateInfo,
) -> WindowResult<Box<dyn PlatformWindow>> {
    match get_linux_display_type() {
        __LinuxDisplay::X11 => todo!("X11 support is not yet implemented"),
        __LinuxDisplay::Wayland => Ok(Box::new(WlWindow::new(create_info)?)),
    }
}
