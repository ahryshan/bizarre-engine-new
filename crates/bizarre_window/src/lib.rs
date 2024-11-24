#![feature(generic_arg_infer)]

#[cfg(all(target_os = "linux"))]
mod linux;

mod platform_window;
mod window;
mod window_create_info;

pub mod window_error;
pub mod window_events;
pub mod window_manager;

pub use platform_window::PlatformWindow;
pub use window::{Window, WindowHandle, WindowMode, WindowStatus};
pub use window_create_info::WindowCreateInfo;

#[cfg(feature = "wayland")]
pub fn get_wayland_display_ptr() -> *const () {
    use wayland_client::Proxy;

    let ctx = linux::wayland::wl_context::WL_CONTEXT.read().unwrap();
    ctx.display.id().as_ptr() as *const ()
}

#[cfg(feature = "wayland")]
pub fn get_wayland_test_surface_ptr() -> *const () {
    use wayland_client::Proxy;

    // This is needed to make sure the wayland context has already
    // been set before trying to get test surface
    drop(linux::wayland::wl_context::WL_CONTEXT.try_read());

    linux::wayland::wl_context::WL_TEST_SURFACE
        .get()
        .expect("Wayland context must be set before trying to get a test surface")
        .id()
        .as_ptr() as *const ()
}
