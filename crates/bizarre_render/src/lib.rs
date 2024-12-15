#![feature(slice_pattern)]
#![feature(variant_count)]

extern crate vk_mem as vma;

mod debug_messenger;
mod device;
mod image;
mod instance;

pub mod antialiasing;
pub mod asset_manager;
pub mod buffer;
pub mod ecs;
pub mod material;
pub mod mesh;
pub mod present_target;
pub mod render_pass;
pub mod render_target;
pub mod renderer;
pub mod scene;
pub mod shader;
pub mod submitter;
pub mod vertex;
