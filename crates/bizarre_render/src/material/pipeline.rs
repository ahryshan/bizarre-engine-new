use core::slice::SlicePattern;
use std::{ffi::CStr, fs::File, io::Read, ops::Deref, path::Path};

use ash::vk;
use bizarre_core::Handle;
use bizarre_log::core_warn;
use thiserror::Error;

use crate::{
    device::LogicalDevice,
    shader::{load_shader, ShaderError, ShaderStage},
};

use super::{
    material_binding::{bindings_into_layouts, MaterialBinding},
    pipeline_features::{PipelineFeatureFlags, VulkanPipelineFeatures},
};

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Failed to compile shader: {0}")]
    ShaderError(#[from] ShaderError),
    #[error(transparent)]
    VkError(#[from] vk::Result),
}

pub type PipelineResult<T> = Result<T, PipelineError>;

pub type PipelineHandle = Handle<VulkanPipeline>;

#[derive(Debug, Clone)]
pub struct ShaderStageDefinition {
    pub path: String,
    pub stage: ShaderStage,
}

#[derive(Debug, Clone)]
pub struct VulkanPipelineRequirements<'a> {
    pub features: VulkanPipelineFeatures,
    pub bindings: Vec<MaterialBinding>,
    pub stage_definitions: Vec<ShaderStageDefinition>,
    pub base_pipeline: Option<&'a VulkanPipeline>,
    pub vertex_bindings: Vec<vk::VertexInputBindingDescription>,
    pub vertex_attributes: Vec<vk::VertexInputAttributeDescription>,
    pub samples: vk::SampleCountFlags,
    pub color_attachment_formats: Vec<vk::Format>,
    pub input_attachment_indices: Vec<u32>,
    pub depth_attachment_format: vk::Format,
}

#[derive(Debug)]
pub struct VulkanPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl VulkanPipeline {
    pub fn from_requirements(
        requirements: &VulkanPipelineRequirements,
        base_pipeline: Option<vk::Pipeline>,
        device: &LogicalDevice,
    ) -> PipelineResult<Self> {
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_binding_descriptions = &requirements.vertex_bindings;
        let vertex_input_attributes = &requirements.vertex_attributes;

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_input_attributes);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(requirements.features.primitive_topology.into())
            .primitive_restart_enable(false);

        let scissors = [vk::Rect2D::default()];
        let viewports = [vk::Viewport::default()];

        let viewport_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(requirements.features.polygon_mode.into())
            .line_width(1.0)
            .cull_mode(requirements.features.culling.into())
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(requirements.samples);

        let color_blend_attachments = {
            let mut attachments = Vec::with_capacity(requirements.color_attachment_formats.len());
            let mut blend_state = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA);

            let feature_flags = requirements.features.flags;

            if feature_flags.intersects(PipelineFeatureFlags::BLEND_MASK) {
                blend_state = blend_state.blend_enable(true);

                if feature_flags.contains(PipelineFeatureFlags::BLEND_COLOR) {
                    blend_state = blend_state
                        .color_blend_op(vk::BlendOp::ADD)
                        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA);
                } else {
                    blend_state = blend_state
                        .color_blend_op(vk::BlendOp::MAX)
                        .src_color_blend_factor(vk::BlendFactor::ONE)
                        .dst_color_blend_factor(vk::BlendFactor::ZERO);
                }

                if feature_flags.contains(PipelineFeatureFlags::BLEND_ALPHA) {
                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::ADD)
                        .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA);
                } else {
                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::MAX)
                        .src_alpha_blend_factor(vk::BlendFactor::ONE)
                        .dst_alpha_blend_factor(vk::BlendFactor::ZERO);
                }

                if feature_flags.contains(PipelineFeatureFlags::BLEND_ADD) {
                    #[cfg(debug_assertions)]
                    if feature_flags.intersects(PipelineFeatureFlags::BLEND_COLOR_ALPHA) {
                        core_warn!(
                            "Pipeline is being created with BLEND_ADD and BLEND_COLOR/BLEND_ALPHA at the same time. Additional blending is being used"
                        );
                    }

                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::ADD)
                        .src_alpha_blend_factor(vk::BlendFactor::ONE)
                        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                        .color_blend_op(vk::BlendOp::ADD)
                        .src_color_blend_factor(vk::BlendFactor::ONE)
                        .dst_color_blend_factor(vk::BlendFactor::ONE);
                }
            } else {
                blend_state = blend_state.blend_enable(false)
            }

            for _ in 0..requirements.color_attachment_formats.len() {
                attachments.push(blend_state.clone());
            }

            attachments
        };

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let bindings = requirements.bindings.to_vec().into();

        let set_layouts = bindings_into_layouts(&bindings)?;

        let layout = {
            let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
            unsafe { device.create_pipeline_layout(&layout_info, None)? }
        };

        let (modules, stages): (Vec<_>, Vec<_>) = requirements
            .stage_definitions
            .iter()
            .map(|ShaderStageDefinition { path, stage }| {
                let code = load_shader(Path::new(path), *stage)?;

                let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

                let module = unsafe { device.create_shader_module(&create_info, None)? };

                let stage = vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::from(*stage))
                    .module(module)
                    .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") });

                Ok((module, stage))
            })
            .collect::<Result<Vec<(_, _)>, ShaderError>>()?
            .into_iter()
            .unzip();

        let mut depth_stencil_info =
            vk::PipelineDepthStencilStateCreateInfo::default().depth_bounds_test_enable(false);

        if requirements.features.flags & PipelineFeatureFlags::DEPTH_MASK
            != PipelineFeatureFlags::empty()
        {
            depth_stencil_info = depth_stencil_info.depth_compare_op(vk::CompareOp::LESS);

            if requirements
                .features
                .flags
                .contains(PipelineFeatureFlags::DEPTH_TEST)
            {
                depth_stencil_info = depth_stencil_info.depth_test_enable(true);
            }
            if requirements
                .features
                .flags
                .contains(PipelineFeatureFlags::DEPTH_WRITE)
            {
                depth_stencil_info = depth_stencil_info.depth_write_enable(true);
            }
            if requirements
                .features
                .flags
                .contains(PipelineFeatureFlags::STENCIL_TEST)
            {
                depth_stencil_info = depth_stencil_info.stencil_test_enable(true);
            }
        }

        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&requirements.color_attachment_formats)
            .depth_attachment_format(requirements.depth_attachment_format);

        let mut attachment_index_info = vk::RenderingInputAttachmentIndexInfoKHR::default()
            .color_attachment_input_indices(&requirements.input_attachment_indices);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .depth_stencil_state(&depth_stencil_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .flags(vk::PipelineCreateFlags::DESCRIPTOR_BUFFER_EXT)
            .layout(layout)
            .push_next(&mut pipeline_rendering_info)
            .push_next(&mut attachment_index_info);

        let pipeline_create_info = if let Some(pipeline) = base_pipeline {
            pipeline_create_info.base_pipeline_handle(pipeline)
        } else {
            pipeline_create_info
        };

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .map_err(|(_, e)| e)?
        };

        for module in modules {
            unsafe {
                device.destroy_shader_module(module, None);
            }
        }

        Ok(VulkanPipeline {
            pipeline: pipeline[0],
            layout,
            set_layouts,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_pipeline_layout(self.layout, None);
            self.layout = vk::PipelineLayout::null();

            for layout in self.set_layouts.iter_mut() {
                device.destroy_descriptor_set_layout(*layout, None);
                *layout = vk::DescriptorSetLayout::null();
            }

            device.destroy_pipeline(self.pipeline, None);
            self.pipeline = vk::Pipeline::null();
        }
    }
}
