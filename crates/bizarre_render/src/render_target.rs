use core::slice;
use std::ptr::addr_of;

use ash::vk::{self};
use bizarre_core::Handle;
use bizarre_log::{core_error, core_fatal, core_info};
use nalgebra_glm::UVec2;

use crate::{
    device::LogicalDevice,
    image::{VulkanImage, VulkanImageView},
    material::descriptor_buffer::DescriptorBuffer,
    vulkan_context::{get_device, get_instance},
};

pub type RenderTargetHandle = Handle<SwapchainRenderTarget>;

pub trait RenderFunction = FnOnce(&'static LogicalDevice, vk::CommandBuffer) -> RenderingResult<()>;

pub struct RenderData {
    pub in_flight_fence: vk::Fence,
    pub render_ready: vk::Semaphore,
    pub cmd_buffer: vk::CommandBuffer,
    pub framebuffer: vk::Framebuffer,
    pub extent: vk::Extent2D,
    pub image: vk::Image,
}

pub struct RenderData2 {
    pub in_flight_fence: vk::Fence,
    pub render_complete: vk::Semaphore,
    pub cmd_buffer: vk::CommandBuffer,
    pub size: UVec2,
}

pub struct SwapchainRenderTarget {
    targets: Vec<ImageRenderTarget>,
    curr_image_index: usize,
}

type RenderingResult<T> = Result<T, vk::Result>;

impl SwapchainRenderTarget {
    pub fn new(
        device: &LogicalDevice,
        size: UVec2,
        cmd_pool: vk::CommandPool,
        samples: vk::SampleCountFlags,
        image_count: u32,
    ) -> RenderingResult<Self> {
        let targets = (0..image_count)
            .map(|_| ImageRenderTarget::new(device, cmd_pool, size, samples))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            targets,
            curr_image_index: 0,
        })
    }

    pub fn resize(&mut self, size: UVec2) -> RenderingResult<()> {
        self.current_target_mut().resize(size)
    }

    pub fn output_image(&self) -> &VulkanImage {
        self.current_target().output_image()
    }

    pub fn composition_attachments(&self) -> Vec<&VulkanImage> {
        self.current_target().composition_attachments()
    }

    pub fn cmd_buffer(&self) -> vk::CommandBuffer {
        self.current_target().render_cmd_buffer
    }

    pub fn render_complete_semaphore(&self) -> vk::Semaphore {
        self.current_target().render_complete
    }

    pub fn in_flight_fence(&self) -> vk::Fence {
        self.current_target().in_flight_fence
    }

    pub fn render<D, C>(
        &mut self,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        deferred: D,
        composition: C,
    ) -> RenderingResult<()>
    where
        D: RenderFunction,
        C: RenderFunction,
    {
        let device = get_device();
        let cmd_buffer = self.current_target().render_cmd_buffer;

        self.begin_rendering(device)?;

        deferred(device, cmd_buffer)?;

        self.start_composition_pass(device)?;

        composition(device, cmd_buffer)?;

        self.end_rendering(device);

        self.prepare_transfer(device);

        self.submit_render(device)?;

        Ok(())
    }

    pub fn next_frame(&mut self) {
        self.curr_image_index = (self.curr_image_index + 1) % self.targets.len();
    }

    pub fn begin_rendering(&mut self, device: &LogicalDevice) -> RenderingResult<RenderData2> {
        self.current_target_mut().begin_rendering(device)
    }

    pub fn start_composition_pass(&mut self, device: &LogicalDevice) -> RenderingResult<()> {
        self.current_target_mut().start_composition_pass(device)
    }

    pub fn end_rendering(&mut self, device: &LogicalDevice) {
        self.current_target_mut().end_rendering(device)
    }

    pub fn prepare_transfer(&mut self, device: &LogicalDevice) {
        self.current_target_mut().prepare_transfer(device)
    }

    pub fn submit_render(&mut self, device: &LogicalDevice) -> RenderingResult<()> {
        self.current_target_mut().submit_render(device)
    }

    fn current_target_mut(&mut self) -> &mut ImageRenderTarget {
        &mut self.targets[self.curr_image_index]
    }

    fn current_target(&self) -> &ImageRenderTarget {
        &self.targets[self.curr_image_index]
    }
}

pub struct ImageRenderTarget {
    pub render_cmd_buffer: vk::CommandBuffer,
    pub in_flight_fence: vk::Fence,
    pub render_complete: vk::Semaphore,

    pub color_attachment: VulkanImage,
    pub normals_attachment: VulkanImage,
    pub position_depth_attachment: VulkanImage,
    pub depth_image: VulkanImage,

    pub output_attachment: VulkanImage,
    pub resolve_attachment: Option<VulkanImage>,
    pub size: UVec2,
}

impl ImageRenderTarget {
    pub fn new(
        device: &LogicalDevice,
        cmd_pool: vk::CommandPool,
        size: UVec2,
        samples: vk::SampleCountFlags,
    ) -> RenderingResult<Self> {
        let in_flight_fence = unsafe {
            let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            device.create_fence(&create_info, None)?
        };

        device.set_object_debug_name(in_flight_fence, "ImageRenderTarget::in_flight_fence");

        let cmd_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(cmd_pool)
                .command_buffer_count(1)
                .level(vk::CommandBufferLevel::PRIMARY);

            unsafe {
                device
                    .allocate_command_buffers(&allocate_info)?
                    .first()
                    .unwrap()
                    .to_owned()
            }
        };

        device.set_object_debug_name(cmd_buffer, "ImageRenderTarget::render_cmd_buffer");

        let color_attachment = VulkanImage::attachment_image(size, samples)?;
        let normals_attachment = VulkanImage::attachment_image(size, samples)?;
        let position_depth_attachment = VulkanImage::attachment_image(size, samples)?;
        let depth_attachment = VulkanImage::depth_image(size, samples)?;

        let (output_attachment, resolve_image) = if samples != vk::SampleCountFlags::TYPE_1 {
            todo!("Multisampling is yet to be implemented")
        } else {
            (VulkanImage::output_image(size)?, None)
        };

        let render_ready = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&create_info, None)? }
        };

        device.set_object_debug_name(render_ready, "ImageRenderTarget::render_complete");

        Ok(Self {
            render_cmd_buffer: cmd_buffer,
            in_flight_fence,
            color_attachment,
            normals_attachment,
            position_depth_attachment,
            depth_image: depth_attachment,
            resolve_attachment: resolve_image,
            size,
            output_attachment,
            render_complete: render_ready,
        })
    }

    pub fn composition_attachments(&self) -> Vec<&VulkanImage> {
        [
            &self.color_attachment,
            &self.normals_attachment,
            &self.position_depth_attachment,
        ]
        .to_vec()
    }

    pub fn output_image(&self) -> &VulkanImage {
        self.resolve_attachment
            .as_ref()
            .unwrap_or(&self.output_attachment)
    }

    pub fn output_image_mut(&mut self) -> &mut VulkanImage {
        self.resolve_attachment
            .as_mut()
            .unwrap_or(&mut self.output_attachment)
    }

    pub fn resize(&mut self, size: UVec2) -> RenderingResult<()> {
        if size <= self.size {
            return Ok(());
        }

        [
            &mut self.color_attachment,
            &mut self.normals_attachment,
            &mut self.position_depth_attachment,
            &mut self.output_attachment,
            &mut self.depth_image,
        ]
        .iter_mut()
        .map(|image| image.resize(size))
        .collect::<Result<(), _>>()?;

        if let Some(image) = &mut self.resolve_attachment {
            image.resize(size)?;
        }

        self.size = size;

        Ok(())
    }

    pub fn begin_rendering(&mut self, device: &LogicalDevice) -> RenderingResult<RenderData2> {
        unsafe {
            device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?;

            device.begin_command_buffer(self.render_cmd_buffer, &Default::default())?;

            self.transition_images_to_deferred(device);

            device.cmd_set_scissor(
                self.render_cmd_buffer,
                0,
                &[vk::Rect2D {
                    extent: vk::Extent2D {
                        width: self.size.x,
                        height: self.size.y,
                    },
                    ..Default::default()
                }],
            );

            device.cmd_set_viewport(
                self.render_cmd_buffer,
                0,
                &[vk::Viewport {
                    height: -(self.size.y as f32),
                    width: self.size.x as f32,
                    x: 0.0,
                    y: self.size.y as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );

            let clear_depth_value = vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            };

            let color_attachments = [
                &self.color_attachment,
                &self.normals_attachment,
                &self.position_depth_attachment,
            ]
            .map(|image| {
                vk::RenderingAttachmentInfo::default()
                    .image_view(image.image_view)
                    .image_layout(image.image_layout)
                    .clear_value(
                        (vk::ClearValue {
                            color: vk::ClearColorValue { float32: [0.3; 4] },
                        })
                        .clone(),
                    )
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
            });

            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_image.image_view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .clear_value(clear_depth_value)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE);

            let rendering_info = vk::RenderingInfo::default()
                .render_area(vk::Rect2D {
                    extent: vk::Extent2D {
                        width: self.size.x,
                        height: self.size.y,
                    },
                    offset: vk::Offset2D::default(),
                })
                .color_attachments(&color_attachments)
                .depth_attachment(&depth_attachment)
                .layer_count(1);

            device.cmd_begin_rendering(self.render_cmd_buffer, &rendering_info);
        }

        let render_data = RenderData2 {
            in_flight_fence: self.in_flight_fence,
            render_complete: self.render_complete,
            cmd_buffer: self.render_cmd_buffer,
            size: self.size,
        };

        Ok(render_data)
    }

    pub fn start_composition_pass(&mut self, device: &LogicalDevice) -> RenderingResult<()> {
        unsafe {
            device.cmd_end_rendering(self.render_cmd_buffer);

            self.transition_images_to_composition(device);

            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 0.0, 0.0, 1.0],
                },
            };

            let color_input_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.output_attachment.image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(clear_color);

            let normals_input_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.normals_attachment.image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(clear_color);

            let position_depth_input_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.position_depth_attachment.image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(clear_color);

            let color_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(self.output_attachment.image_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_color);

            let color_attachments = [
                color_input_attachment,
                normals_input_attachment,
                position_depth_input_attachment,
                color_attachment,
            ];

            let rendering_info = vk::RenderingInfo::default()
                .color_attachments(&color_attachments)
                .layer_count(1)
                .render_area(vk::Rect2D {
                    extent: vk::Extent2D {
                        width: self.size.x,
                        height: self.size.y,
                    },
                    offset: vk::Offset2D { x: 0, y: 0 },
                });

            device.cmd_begin_rendering(self.render_cmd_buffer, &rendering_info);
        }

        Ok(())
    }

    pub fn end_rendering(&mut self, device: &LogicalDevice) {
        unsafe { device.cmd_end_rendering(self.render_cmd_buffer) }
    }

    pub fn prepare_transfer(&mut self, device: &LogicalDevice) {
        unsafe {
            let cmd = self.render_cmd_buffer;

            let image_barrier = self.output_image_mut().image_barrier(
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            );

            let barriers = [image_barrier];

            let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);

            device.cmd_pipeline_barrier2(cmd, &dep_info);
        }
    }

    pub fn submit_render(&self, device: &LogicalDevice) -> RenderingResult<()> {
        unsafe { device.end_command_buffer(self.render_cmd_buffer) };

        let cmd = [self.render_cmd_buffer];
        let signal_semaphores = [self.render_complete];

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&cmd)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device.reset_fences(&[self.in_flight_fence])?;
            device.queue_submit(device.graphics_queue, &[submit_info], self.in_flight_fence)?;
        }

        Ok(())
    }

    fn transition_images_to_deferred(&mut self, device: &LogicalDevice) {
        let attachment_barriers = [
            &mut self.color_attachment,
            &mut self.normals_attachment,
            &mut self.position_depth_attachment,
        ]
        .map(|image| unsafe {
            image.image_barrier(
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::empty(),
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            )
            // image.image_barrier(
            //     vk::PipelineStageFlags2::TOP_OF_PIPE,
            //     vk::AccessFlags2::empty(),
            //     vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            //     vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            //     vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            // )
        });

        let depth_barrier = unsafe {
            self.depth_image.image_barrier(
                vk::PipelineStageFlags2::TOP_OF_PIPE,
                vk::AccessFlags2::empty(),
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS,
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            )
        };

        let barriers = [&attachment_barriers[..], &[depth_barrier][..]].concat();

        let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);

        unsafe { device.cmd_pipeline_barrier2(self.render_cmd_buffer, &dep_info) };
    }

    fn transition_images_to_composition(&mut self, device: &LogicalDevice) {
        let image_barriers = [
            &mut self.color_attachment,
            &mut self.normals_attachment,
            &mut self.position_depth_attachment,
        ]
        .map(|image| unsafe {
            image.image_barrier(
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            )
        });

        let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&image_barriers);

        unsafe { device.cmd_pipeline_barrier2(self.render_cmd_buffer, &dependency_info) };
    }
}

impl Drop for ImageRenderTarget {
    fn drop(&mut self) {
        let device = get_device();

        unsafe {
            if let Err(err) = device.wait_for_fences(&[self.in_flight_fence], true, u64::MAX) {
                core_error!("ImageRenderTarget::drop: failed to wait for fences: {err}");
            }
            device.destroy_fence(self.in_flight_fence, None);
            device.destroy_semaphore(self.render_complete, None);
        }
    }
}
