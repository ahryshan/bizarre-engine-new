use ash::vk;
use bizarre_core::Handle;
use nalgebra_glm::UVec2;

use crate::{
    device::VulkanDevice,
    image::AttachmentImage,
    render_pass::{RenderPassAttachment, RenderPassHandle, VulkanRenderPass},
};

pub type RenderTargetHandle = Handle<ImageRenderTarget>;

pub trait RenderTarget {
    fn render_pass(&self) -> vk::RenderPass;
    fn get_render_data(&self) -> RenderData;
    fn output_image(&self) -> &AttachmentImage;
    fn render_complete_semaphore(&self) -> vk::Semaphore;
    fn next_frame(&mut self);

    fn destroy(&mut self, device: &VulkanDevice);
}

pub struct RenderData {
    pub in_flight_fence: vk::Fence,
    pub render_ready: vk::Semaphore,
    pub cmd_buffer: vk::CommandBuffer,
    pub framebuffer: vk::Framebuffer,
    pub extent: vk::Extent2D,
    pub image: vk::Image,
}

pub struct SwapchainRenderTarget {
    targets: Vec<ImageRenderTarget>,
    curr_image_index: usize,
}

impl SwapchainRenderTarget {
    pub fn new(
        device: &VulkanDevice,
        extent: UVec2,
        cmd_pool: vk::CommandPool,
        render_pass: &VulkanRenderPass,
        image_count: u32,
    ) -> Result<Self, vk::Result> {
        let targets = (0..image_count)
            .map(|_| ImageRenderTarget::new(device, extent, cmd_pool, render_pass))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            targets,
            curr_image_index: 0,
        })
    }
}

impl RenderTarget for SwapchainRenderTarget {
    fn render_pass(&self) -> vk::RenderPass {
        self.targets[0].render_pass
    }

    fn get_render_data(&self) -> RenderData {
        let ImageRenderTarget {
            cmd_buffer,
            in_flight_fence,
            render_complete: render_ready,
            extent,
            framebuffer,
            images,
            output_image_index,
            ..
        } = &self.targets[self.curr_image_index];

        let image = images[*output_image_index].image;

        RenderData {
            in_flight_fence: *in_flight_fence,
            render_ready: *render_ready,
            cmd_buffer: *cmd_buffer,
            extent: vk::Extent2D {
                width: extent.x,
                height: extent.y,
            },
            framebuffer: *framebuffer,
            image,
        }
    }

    fn output_image(&self) -> &AttachmentImage {
        let target = &self.targets[self.curr_image_index];
        &target.images[target.output_image_index]
    }

    fn render_complete_semaphore(&self) -> vk::Semaphore {
        self.targets[self.curr_image_index].render_complete
    }

    fn next_frame(&mut self) {
        self.curr_image_index += 1;
        self.curr_image_index %= self.targets.len();
    }

    fn destroy(&mut self, device: &VulkanDevice) {
        self.targets
            .drain(..)
            .for_each(|mut target| target.destroy(device));
    }
}

pub struct ImageRenderTarget {
    pub cmd_buffer: vk::CommandBuffer,
    pub in_flight_fence: vk::Fence,
    pub render_complete: vk::Semaphore,
    pub images: Vec<AttachmentImage>,
    pub extent: UVec2,
    pub framebuffer: vk::Framebuffer,
    pub render_pass: vk::RenderPass,
    pub output_image_index: usize,
}

impl ImageRenderTarget {
    pub fn new(
        device: &VulkanDevice,
        size: UVec2,
        cmd_pool: vk::CommandPool,
        render_pass: &VulkanRenderPass,
    ) -> Result<Self, vk::Result> {
        let in_flight_fence = unsafe {
            let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            device.create_fence(&create_info, None)?
        };

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

        let images = render_pass
            .attachments
            .iter()
            .map(|attachment| {
                let (format, usage, aspect_mask, samples) = match attachment {
                    RenderPassAttachment::Resolve => (
                        vk::Format::R8G8B8A8_SRGB,
                        vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                        vk::ImageAspectFlags::COLOR,
                        vk::SampleCountFlags::TYPE_1,
                    ),
                    RenderPassAttachment::Output(samples) => (
                        vk::Format::R8G8B8A8_SRGB,
                        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
                        vk::ImageAspectFlags::COLOR,
                        *samples,
                    ),
                    RenderPassAttachment::Color(samples) => (
                        vk::Format::R8G8B8A8_SRGB,
                        vk::ImageUsageFlags::COLOR_ATTACHMENT
                            | vk::ImageUsageFlags::INPUT_ATTACHMENT
                            | vk::ImageUsageFlags::SAMPLED,
                        vk::ImageAspectFlags::COLOR,
                        *samples,
                    ),
                    RenderPassAttachment::DepthStencil(samples) => (
                        vk::Format::D32_SFLOAT,
                        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                            | vk::ImageUsageFlags::INPUT_ATTACHMENT
                            | vk::ImageUsageFlags::SAMPLED,
                        vk::ImageAspectFlags::DEPTH,
                        *samples,
                    ),
                };
                AttachmentImage::new(device, size, format, usage, aspect_mask, samples)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let render_ready = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&create_info, None)? }
        };

        let framebuffer = {
            let attachments = images
                .iter()
                .map(|image| image.image_view)
                .collect::<Vec<_>>();

            let create_info = vk::FramebufferCreateInfo::default()
                .attachments(&attachments)
                .render_pass(render_pass.render_pass)
                .width(size.x)
                .height(size.y)
                .layers(1);

            unsafe { device.create_framebuffer(&create_info, None)? }
        };

        let output_image_index = if render_pass.msaa() {
            render_pass
                .attachments
                .iter()
                .position(|element| match element {
                    RenderPassAttachment::Resolve => true,
                    _ => false,
                })
                .unwrap()
        } else {
            render_pass
                .attachments
                .iter()
                .position(|element| match element {
                    RenderPassAttachment::Output(..) => true,
                    _ => false,
                })
                .unwrap()
        };

        Ok(Self {
            cmd_buffer,
            in_flight_fence,
            images,
            extent: size,
            render_complete: render_ready,
            framebuffer,
            render_pass: render_pass.render_pass,
            output_image_index,
        })
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
            self.images
                .drain(..)
                .for_each(|mut image| image.destroy(device));
            device.destroy_semaphore(self.render_complete, None);
            device.destroy_fence(self.in_flight_fence, None);
        }
    }
}
