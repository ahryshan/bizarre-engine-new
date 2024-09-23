#![feature(generic_arg_infer)]

#[cfg(all(target_os = "linux"))]
mod linux;

mod platform_window;
mod window;
mod window_action;
mod window_create_info;
mod window_ecs;

pub mod window_error;
pub mod window_events;
pub mod window_manager;

pub use platform_window::PlatformWindow;
pub use window::{WindowHandle, WindowMode, WindowStatus};
pub use window_action::WindowAction;
pub use window_create_info::WindowCreateInfo;
