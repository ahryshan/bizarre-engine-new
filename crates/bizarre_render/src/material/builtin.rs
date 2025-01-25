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
    with_basic_deferred(|_| {})
}

pub fn with_basic_deferred<'a, F>(f: F) -> Material
where
    F: Fn(&mut VulkanPipelineRequirements),
{
    let device = get_device();

    let bindings = base_scene_bindings();

    let mut req = VulkanPipelineRequirements {
        features: VulkanPipelineFeatures {
            flags: PipelineFeatureFlags::DEPTH_TEST | PipelineFeatureFlags::DEPTH_WRITE,
            culling: CullMode::Back,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        },
        bindings,
        stage_definitions: vec![
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
        vertex_bindings: Vertex::bindings().to_vec(),
        vertex_attributes: Vertex::attributes().to_vec(),
        samples: vk::SampleCountFlags::TYPE_1,
        color_attachment_formats: vec![COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT],
        input_attachment_indices: vec![
            vk::ATTACHMENT_UNUSED,
            vk::ATTACHMENT_UNUSED,
            vk::ATTACHMENT_UNUSED,
        ],
        depth_attachment_format: DEPTH_FORMAT,
    };

    f(&mut req);

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();
    Material::new(pipeline, &[])
}

pub fn basic_composition() -> Material {
    let device = get_device();

    let req = VulkanPipelineRequirements {
        features: Default::default(),
        bindings: vec![
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
        stage_definitions: vec![
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
        color_attachment_formats: vec![COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT, COLOR_FORMAT],
        input_attachment_indices: vec![0, 1, 2, vk::ATTACHMENT_UNUSED],
        depth_attachment_format: DEPTH_FORMAT,
    };

    let pipeline = VulkanPipeline::from_requirements(&req, None, device).unwrap();

    Material::new(pipeline, &[])
}
