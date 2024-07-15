#[cfg(all(target_os = "linux"))]
mod linux;

mod window;
mod window_action;
mod window_create_info;

pub mod window_events;

#[cfg(target_os = "linux")]
pub use linux::linux_window::LinuxWindow as Window;

pub use window::{WindowMode, WindowTrait};
pub use window_action::WindowAction;
pub use window_create_info::WindowCreateInfo;
