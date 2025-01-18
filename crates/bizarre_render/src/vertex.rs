use std::mem::offset_of;

use ash::vk;
use nalgebra_glm::Vec3;

#[repr(C, align(4))]
#[derive(Clone, Debug, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub _pad0: f32,
    pub normal: Vec3,
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
        &[
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
        ]
    }
}
