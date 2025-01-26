use core::fmt::Debug;
use std::ffi::{CStr, CString};

use ash::vk;
use bizarre_log::{core_info, core_trace};
use nalgebra_glm::UVec2;
use thiserror::Error;

use bizarre_core::Handle;
use bizarre_ecs::prelude::Resource;

use crate::{
    antialiasing::Antialiasing,
    buffer::GpuBuffer,
    device::logical_device::DeviceError,
    image::VulkanImage,
    instance::InstanceError,
    material::{
        builtin::basic_composition,
        descriptor_buffer::{self, DescriptorBuffer},
        material_instance::{MaterialInstance, MaterialInstanceHandle},
        pipeline::PipelineError,
        Material, MaterialHandle,
    },
    present_target::{PresentData, PresentResult, PresentTargetHandle},
    render_assets::{AssetStore, RenderAssets},
    render_target::RenderTargetHandle,
    scene::{object_pass::SceneObjectPass, IndirectIterItem, Scene, SceneUniform},
    submitter::RenderPackage,
    vulkan_context::{get_device, get_instance},
};

#[derive(Resource)]
pub struct VulkanRenderer {
    image_count: u32,
    current_frame: usize,
    swapchain_loader: ash::khr::swapchain::Device,
    antialiasing: Antialiasing,

    uniform_buffers: DescriptorBuffer,
    curr_uniform_index: usize,

    textures: DescriptorBuffer,
    curr_texture_index: usize,

    input_attachments: DescriptorBuffer,
    curr_input_index: usize,

    basic_composition: Material,
    basic_composition_instance: MaterialInstance,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error(transparent)]
    VulkanError(#[from] vk::Result),
    #[error("Failed to create a `VulkanRenderer`: {0}")]
    CreateError(#[from] RendererCreateError),
    #[error(transparent)]
    PipelineError(#[from] PipelineError),
    #[error("Invalid render target")]
    InvalidRenderTarget,
    #[error("Render must be skipped")]
    RenderSkipped,
}
#[derive(Error, Debug)]
pub enum RendererCreateError {
    #[error(transparent)]
    InstanceError(#[from] InstanceError),
    #[error(transparent)]
    DeviceError(#[from] DeviceError),
}

const IMAGE_COUNT: usize = 4;
const UNIFORM_DESCRIPTOR_BUFFER_LEN: usize = 32;
const TEXTURE_DESCRIPTOR_BUFFER_LEN: usize = 32;
const INPUT_ATTACHMENT_BUFFER_LEN: usize = 32;

const fn descriptor_buffer_len(descriptor_type: vk::DescriptorType) -> usize {
    match descriptor_type {
        vk::DescriptorType::UNIFORM_BUFFER => UNIFORM_DESCRIPTOR_BUFFER_LEN,
        vk::DescriptorType::COMBINED_IMAGE_SAMPLER => TEXTURE_DESCRIPTOR_BUFFER_LEN,
        vk::DescriptorType::INPUT_ATTACHMENT => INPUT_ATTACHMENT_BUFFER_LEN,
        _ => panic!("Unsupported descriptor buffer type"),
    }
}

pub type RenderResult<T> = Result<T, RenderError>;

impl VulkanRenderer {
    pub fn new() -> RenderResult<Self> {
        let instance = get_instance();
        let device = get_device();

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        let uniform_buffers = DescriptorBuffer::uniform_buffers(
            descriptor_buffer_len(vk::DescriptorType::UNIFORM_BUFFER) * IMAGE_COUNT,
        )?;

        device.set_object_debug_name(uniform_buffers.buffer(), "renderer_uniforms");

        let textures = DescriptorBuffer::textures(
            descriptor_buffer_len(vk::DescriptorType::COMBINED_IMAGE_SAMPLER) * IMAGE_COUNT,
        )?;

        device.set_object_debug_name(textures.buffer(), "renderer_textures");

        let input_attachments = DescriptorBuffer::new(
            INPUT_ATTACHMENT_BUFFER_LEN * IMAGE_COUNT,
            &[vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)],
            vk::BufferUsageFlags::empty(),
        )?;

        device.set_object_debug_name(textures.buffer(), "renderer_input_attachments");

        let basic_composition_mat = basic_composition();
        let basic_composition_instance =
            MaterialInstance::new(MaterialHandle::from_raw(0usize), &basic_composition_mat)
                .unwrap();

        Ok(Self {
            // TODO: Make it dynamic and/or configurable
            image_count: IMAGE_COUNT as u32,
            curr_uniform_index: 0,
            curr_texture_index: 0,
            curr_input_index: 0,
            current_frame: 0,
            antialiasing: Antialiasing::None,
            swapchain_loader,
            uniform_buffers,
            textures,
            input_attachments,

            basic_composition: basic_composition_mat,
            basic_composition_instance,
        })
    }

    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.image_count as usize;
        self.curr_uniform_index = 0;
        self.curr_texture_index = 0;
        self.curr_input_index = 0;
    }

    pub fn render_to_target(
        &mut self,
        assets: &mut RenderAssets,
        render_target: RenderTargetHandle,
        render_extent: UVec2,
        render_package: RenderPackage,
    ) -> RenderResult<()> {
        if render_extent.x == 0 || render_extent.y == 0 {
            return Err(RenderError::RenderSkipped);
        }

        let RenderPackage {
            scene: scene_handle,
            pov,
        } = render_package;

        let device = get_device();

        // TODO: I'm really sorry for what I've done. But it's safe, I'm promise. I'll fix that later
        let scene = unsafe { &mut *(assets.scenes.get_mut(&scene_handle).unwrap() as *mut Scene) };

        scene.sync_frame_data(&assets.meshes);

        let (indirect_buffer, indirect_iter) = scene.indirect_draw_iterator();
        let (_, scene_ubo_offset) =
            self.add_uniform(scene.scene_ubo(), 0, scene.scene_ubo().size());

        let vertex_buffer = scene.vertex_buffer();
        let index_buffer = scene.index_buffer();

        #[derive(Debug)]
        struct DrawItem {
            mat_handle: MaterialHandle,
            inst_handle: MaterialInstanceHandle,
            pipeline: vk::Pipeline,
            pipeline_layout: vk::PipelineLayout,
            indirect_offset: u64,
            batch_offset: u64,
            batch_range: u64,
            count: u32,
        }

        let deferred_indirects = indirect_iter
            .clone()
            .filter_map(
                |IndirectIterItem {
                     materials,
                     indirect_offset,
                     count,
                     batch_offset,
                     batch_range,
                 }| {
                    let instance_handle = materials[SceneObjectPass::Deferred]?;
                    let (material, instance) = assets.material_with_instance(&instance_handle)?;

                    let pipeline = material.pipeline();

                    Some(DrawItem {
                        mat_handle: instance.material_handle(),
                        inst_handle: instance_handle,
                        pipeline: pipeline.pipeline,
                        pipeline_layout: pipeline.layout,
                        indirect_offset,
                        count,
                        batch_offset,
                        batch_range,
                    })
                },
            )
            .collect::<Vec<_>>();

        let render_target = assets
            .render_targets
            .get_mut(&render_target)
            .ok_or(RenderError::InvalidRenderTarget)?;

        render_target.resize(render_extent);

        render_target.begin_rendering(device)?;

        let cmd_buffer = render_target.cmd_buffer();

        unsafe {
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(cmd_buffer, index_buffer, 0, vk::IndexType::UINT32);
        }

        let mut bound_mat = Handle::null();
        let mut bound_inst = Handle::null();

        let db_device_ext = descriptor_buffer::device_ext();

        let bind_info = [self.uniform_buffers.binding_info()];
        unsafe { db_device_ext.cmd_bind_descriptor_buffers(cmd_buffer, &bind_info) };

        for DrawItem {
            mat_handle,
            inst_handle,
            pipeline,
            pipeline_layout,
            indirect_offset,
            count,
            batch_offset,
            batch_range,
        } in deferred_indirects
        {
            let (_, instance_data_offset) =
                self.add_uniform(scene.instance_data_ubo(), batch_offset, batch_range);

            unsafe {
                db_device_ext.cmd_set_descriptor_buffer_offsets(
                    cmd_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &[0, 0],
                    &[scene_ubo_offset, instance_data_offset],
                );
            }

            let mat_rebind = bound_mat != mat_handle;
            let inst_rebind = mat_rebind || bound_inst != inst_handle;

            if mat_rebind {
                unsafe {
                    device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
                }

                bound_mat = mat_handle;
            }

            if inst_rebind {
                bound_inst = inst_handle;
            }

            unsafe {
                device.cmd_draw_indexed_indirect(
                    cmd_buffer,
                    indirect_buffer.buffer(),
                    indirect_offset,
                    count,
                    size_of::<vk::DrawIndexedIndirectCommand>() as u32,
                )
            }
        }

        render_target.start_composition_pass(device)?;

        unsafe {
            device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.basic_composition.pipeline().pipeline,
            )
        };

        let bind_info = [self.input_attachments.binding_info()];

        let device_ext = descriptor_buffer::device_ext();

        let attachment_offsets = render_target
            .composition_attachments()
            .into_iter()
            .map(|image| self.add_input_attachment(image).1)
            .collect::<Vec<_>>();

        unsafe {
            device_ext.cmd_bind_descriptor_buffers(cmd_buffer, &bind_info);

            device_ext.cmd_set_descriptor_buffer_offsets(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.basic_composition.pipeline().layout,
                0,
                &[0],
                &[attachment_offsets[0]],
            );
        }

        unsafe { device.cmd_draw(cmd_buffer, 6, 1, 0, 0) }

        render_target.end_rendering(device);
        render_target.prepare_transfer(device);
        render_target.submit_render(device)?;

        self.next_frame();
        assets.scenes.get_mut(&scene_handle).unwrap().next_frame();

        Ok(())
    }

    #[allow(unused)]
    #[inline]
    fn add_uniform(
        &mut self,
        buffer: &GpuBuffer,
        buffer_offset: vk::DeviceSize,
        buffer_range: vk::DeviceSize,
    ) -> (usize, vk::DeviceSize) {
        let index = self.current_frame * UNIFORM_DESCRIPTOR_BUFFER_LEN + self.curr_uniform_index;

        let offset = unsafe {
            self.uniform_buffers.set_uniform_buffer_unchecked(
                buffer,
                buffer_offset,
                buffer_range,
                index,
            )
        };

        self.curr_uniform_index += 1;

        (index, offset)
    }

    #[allow(unused)]
    #[inline]
    fn add_texture(
        &mut self,
        texture: &VulkanImage,
        sampler: vk::Sampler,
    ) -> (usize, vk::DeviceSize) {
        let index = self.current_frame * TEXTURE_DESCRIPTOR_BUFFER_LEN + self.curr_texture_index;

        let offset = unsafe { self.textures.set_texture_unchecked(texture, sampler, index) };

        self.curr_texture_index += 1;

        (index, offset)
    }

    #[allow(unused)]
    #[inline]
    fn add_input_attachment(&mut self, texture: &VulkanImage) -> (usize, vk::DeviceSize) {
        let index = self.current_frame * INPUT_ATTACHMENT_BUFFER_LEN + self.curr_input_index;

        let offset = unsafe {
            self.input_attachments
                .set_input_attachment_unchecked(texture, index)
        };

        self.curr_input_index += 1;

        (index, offset)
    }

    pub fn image_count(&self) -> u32 {
        self.image_count
    }

    pub fn antialising(&self) -> Antialiasing {
        self.antialiasing
    }

    pub fn present_to_target(
        &mut self,
        assets: &mut RenderAssets,
        present_target: PresentTargetHandle,
        render_target: RenderTargetHandle,
    ) -> PresentResult<()> {
        let device = get_device();

        unsafe { device.device_wait_idle()? }

        let present_target = assets.present_targets.get_mut(&present_target).unwrap();
        let render_target = assets.render_targets.get_mut(&render_target).unwrap();

        let PresentData {
            cmd_buffer,
            swapchain,
            image_acquired,
            image_ready,
            image_index: index,
            image_ready_fence,
        } = present_target.record_present(device, render_target.output_image())?;

        let swapchains = [swapchain];
        let indices = [index];
        let buffers = [cmd_buffer].into_iter().flatten().collect::<Vec<_>>();

        let cmd_wait = [image_acquired, render_target.render_complete_semaphore()];
        let images_ready = [image_ready];

        let pipeline_stage_masks = [
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
        ];

        unsafe {
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&buffers)
                .signal_semaphores(&images_ready)
                .wait_semaphores(&cmd_wait)
                .wait_dst_stage_mask(&pipeline_stage_masks);

            let submits = [submit_info];

            device.reset_fences(&[image_ready_fence])?;

            device.queue_submit(device.present_queue, &submits, image_ready_fence)?;
        };

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&indices)
            .wait_semaphores(&images_ready);

        unsafe {
            self.swapchain_loader
                .queue_present(device.present_queue, &present_info)?
        };

        render_target.next_frame();

        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            get_device().device_wait_idle();
        }
    }
}
