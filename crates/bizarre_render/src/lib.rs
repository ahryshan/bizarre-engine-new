#![feature(slice_pattern)]
#![feature(variant_count)]
#![feature(trait_alias)]

use ash::vk;

extern crate vk_mem as vma;

pub const COLOR_FORMAT: vk::Format = vk::Format::R32G32B32A32_SFLOAT;
pub const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
pub const OUTPUT_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;
pub const TMP_SAMPLES: vk::SampleCountFlags = vk::SampleCountFlags::TYPE_1;

mod debug_messenger;
mod device;
mod image;
mod instance;
mod macros;
mod vulkan_context;

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
