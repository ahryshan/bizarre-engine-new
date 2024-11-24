use ash::vk;
use bizarre_core::Handle;
use nalgebra_glm::UVec2;

use crate::{device::VulkanDevice, image::VulkanImage, render_pass::RenderPass};

pub type RenderTargetHandle = Handle<ImageRenderTarget>;

pub trait RenderTarget {
    fn render_pass(&self) -> RenderPass;
    fn get_render_data(&self) -> RenderData;
    fn image(&self) -> &VulkanImage;
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
        vk_render_pass: vk::RenderPass,
        render_pass: RenderPass,
        image_count: u32,
    ) -> Result<Self, vk::Result> {
        let targets = (0..image_count)
            .map(|_| ImageRenderTarget::new(device, extent, cmd_pool, render_pass, vk_render_pass))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            targets,
            curr_image_index: 0,
        })
    }
}

impl RenderTarget for SwapchainRenderTarget {
    fn render_pass(&self) -> RenderPass {
        self.targets[0].render_pass
    }

    fn get_render_data(&self) -> RenderData {
        let ImageRenderTarget {
            cmd_buffer,
            in_flight_fence,
            render_complete: render_ready,
            extent,
            framebuffer,
            image,
            ..
        } = &self.targets[self.curr_image_index];

        RenderData {
            in_flight_fence: *in_flight_fence,
            render_ready: *render_ready,
            cmd_buffer: *cmd_buffer,
            extent: vk::Extent2D {
                width: extent.x,
                height: extent.y,
            },
            framebuffer: *framebuffer,
            image: image.image,
        }
    }

    fn image(&self) -> &VulkanImage {
        &self.targets[self.curr_image_index].image
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
    pub image: VulkanImage,
    pub extent: UVec2,
    pub framebuffer: vk::Framebuffer,
    pub render_pass: RenderPass,
}

impl ImageRenderTarget {
    pub fn new(
        device: &VulkanDevice,
        extent: UVec2,
        cmd_pool: vk::CommandPool,
        render_pass: RenderPass,
        vk_render_pass: vk::RenderPass,
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

        let image = VulkanImage::new(device, extent)?;

        let render_ready = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&create_info, None)? }
        };

        let framebuffer = {
            let attachments = [image.image_view];

            let create_info = vk::FramebufferCreateInfo::default()
                .attachments(&attachments)
                .render_pass(vk_render_pass)
                .width(extent.x)
                .height(extent.y)
                .layers(1);

            unsafe { device.create_framebuffer(&create_info, None)? }
        };

        Ok(Self {
            cmd_buffer,
            in_flight_fence,
            image,
            extent,
            render_complete: render_ready,
            framebuffer,
            render_pass,
        })
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
            self.image.destroy(device);
            device.destroy_semaphore(self.render_complete, None);
            device.destroy_fence(self.in_flight_fence, None);
        }
    }
}
