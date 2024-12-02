#![feature(slice_pattern)]

extern crate vk_mem as vma;

mod debug_messenger;
mod device;
mod image;
mod instance;

pub mod antialiasing;
pub mod buffer;
pub mod ecs;
pub mod material;
pub mod present_target;
pub mod render_pass;
pub mod render_target;
pub mod renderer;
pub mod resource_manager;
pub mod shader;
pub mod submitter;
pub mod vertex;
