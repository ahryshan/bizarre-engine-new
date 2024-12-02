use std::mem::offset_of;

use ash::vk;
use nalgebra_glm::Vec3;

#[repr(C)]
pub struct Vertex {
    position: Vec3,
}

impl Vertex {
    pub fn bindings() -> &'static [vk::VertexInputBindingDescription] {
        &[vk::VertexInputBindingDescription {
            binding: 0,
            input_rate: vk::VertexInputRate::VERTEX,
            stride: size_of::<Vertex>() as u32,
            ..Default::default()
        }]
    }

    pub fn attributes() -> &'static [vk::VertexInputAttributeDescription] {
        &[vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, position),
            ..Default::default()
        }]
    }
}
