use ash::vk;

use crate::{
    device::LogicalDevice,
    shader::{ShaderStage, ShaderStageFlags, ShaderStages},
    vertex::Vertex,
    vulkan_context::get_device,
    COLOR_FORMAT, DEPTH_FORMAT,
};

use super::{
    material_binding::{base_scene_bindings, MaterialBinding, MaterialBindingRate},
    pipeline::{ShaderStageDefinition, VulkanPipeline, VulkanPipelineRequirements},
    pipeline_features::{CullMode, PipelineFeatureFlags, PolygonMode, VulkanPipelineFeatures},
    Material,
};

pub fn basic_deferred() -> Material {
    let device = get_device();

    let bindings = base_scene_bindings();

    let req = VulkanPipelineRequirements {
        features: VulkanPipelineFeatures {
            flags: PipelineFeatureFlags::DEPTH_TEST | PipelineFeatureFlags::DEPTH_WRITE,
            culling: CullMode::Back,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        },
        bindings,
        stage_definitions: &[
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_deferred.vert"),
                stage: ShaderStage::Vertex,
            },
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_deferred.frag"),
                stage: ShaderStage::Fragment,
            },
        ],
        base_pipeline: None,
        vertex_bindings: &Vertex::bindings(),
        vertex_attributes: &Vertex::attributes(),
        samples: vk::SampleCountFlags::TYPE_1,
        color_attachment_formats: &[COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT],
        input_attachment_indices: &[
            vk::ATTACHMENT_UNUSED,
            vk::ATTACHMENT_UNUSED,
            vk::ATTACHMENT_UNUSED,
        ],
        depth_attachment_format: DEPTH_FORMAT,
    };

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();
    Material::new(pipeline, &[])
}

pub fn basic_composition() -> Material {
    let device = get_device();

    let req = VulkanPipelineRequirements {
        features: Default::default(),
        bindings: &[
            MaterialBinding {
                binding: 0,
                set: 0,
                binding_rate: MaterialBindingRate::PerFrame,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
                shader_stage_flags: ShaderStageFlags::FRAGMENT,
            },
            MaterialBinding {
                binding: 1,
                set: 0,
                binding_rate: MaterialBindingRate::PerFrame,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
                shader_stage_flags: ShaderStageFlags::FRAGMENT,
            },
            MaterialBinding {
                binding: 2,
                set: 0,
                binding_rate: MaterialBindingRate::PerFrame,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
                shader_stage_flags: ShaderStageFlags::FRAGMENT,
            },
        ],
        stage_definitions: &[
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_composition.vert"),
                stage: ShaderStage::Vertex,
            },
            ShaderStageDefinition {
                path: String::from("assets/shaders/basic_composition.frag"),
                stage: ShaderStage::Fragment,
            },
        ],
        base_pipeline: None,
        vertex_bindings: Default::default(),
        vertex_attributes: Default::default(),
        samples: vk::SampleCountFlags::TYPE_1,
        color_attachment_formats: &[COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT],
        input_attachment_indices: &[0, 1, 2, vk::ATTACHMENT_UNUSED],
        depth_attachment_format: DEPTH_FORMAT,
    };

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();

    Material::new(pipeline, &[])
}
