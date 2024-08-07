#![feature(type_changing_struct_update)]

mod app;
mod default_app_module;
mod ecs_module_buffer;

pub mod app_builder;
pub mod app_event;
pub mod app_state;

pub use app::App;
pub use app_builder::AppBuilder;
