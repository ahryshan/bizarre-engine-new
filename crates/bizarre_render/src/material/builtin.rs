use ash::vk;

use crate::{
    device::LogicalDevice, shader::ShaderKind, vertex::Vertex, vulkan_context::get_device,
    COLOR_FORMAT, DEPTH_FORMAT,
};

use super::{
    pipeline::{ShaderStageDefinition, VulkanPipeline, VulkanPipelineRequirements},
    Material,
};

pub fn basic_deferred() -> Material {
    let device = get_device();

    let req = VulkanPipelineRequirements {
        features: Default::default(),
        bindings: Default::default(),
        stage_definitions: &[
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_deferred.vert"),
                stage: ShaderKind::Vertex,
            },
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_deferred.frag"),
                stage: ShaderKind::Vertex,
            },
        ],
        base_pipeline: None,
        vertex_bindings: &Vertex::bindings(),
        vertex_attributes: &Vertex::attributes(),
        samples: vk::SampleCountFlags::TYPE_1,
        color_attachment_formats: &[COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT],
        depth_attachment_format: DEPTH_FORMAT,
    };

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();
    Material::new(pipeline, &[])
}

pub fn basic_composition() -> Material {
    let device = get_device();

    let req = VulkanPipelineRequirements {
        features: Default::default(),
        bindings: Default::default(),
        stage_definitions: &[ShaderStageDefinition {
            path: String::from("assets/shaders/deferred.vert"),
            stage: ShaderKind::Vertex,
        }],
        base_pipeline: None,
        vertex_bindings: Default::default(),
        vertex_attributes: Default::default(),
        samples: vk::SampleCountFlags::TYPE_1,
        color_attachment_formats: &[COLOR_FORMAT],
        depth_attachment_format: DEPTH_FORMAT,
    };

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();

    Material::new(pipeline, &[])
}
