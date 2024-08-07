#[cfg(all(target_os = "linux"))]
mod linux;

mod window;
mod window_action;
mod window_create_info;
mod window_manager;

pub mod window_error;
pub mod window_events;
pub mod window_systems;

#[cfg(target_os = "linux")]
pub use linux::linux_window::LinuxWindow as Window;

pub use window::{WindowHandle, WindowMode, WindowStatus, WindowTrait};
pub use window_action::WindowAction;
pub use window_create_info::WindowCreateInfo;
