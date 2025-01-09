use std::ops::Deref;

use ash::vk::{self, Handle};
use nalgebra_glm::UVec2;
use vma::Alloc;

use crate::{vulkan_context::get_device, COLOR_FORMAT, DEPTH_FORMAT};

pub struct VulkanImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub allocation: vma::Allocation,
    pub image_layout: vk::ImageLayout,
    pub aspect_mask: vk::ImageAspectFlags,
    pub format: vk::Format,
    pub level_count: u32,
    pub layer_count: u32,
    pub size: UVec2,
}

impl VulkanImage {
    pub fn attachment_image(
        size: UVec2,
        samples: vk::SampleCountFlags,
    ) -> Result<Self, vk::Result> {
        Self::new(
            size,
            COLOR_FORMAT,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            vk::ImageAspectFlags::COLOR,
            samples,
            1,
            1,
        )
    }

    pub fn output_image(size: UVec2) -> Result<Self, vk::Result> {
        Self::new(
            size,
            COLOR_FORMAT,
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR,
            vk::SampleCountFlags::TYPE_1,
            1,
            1,
        )
    }

    pub fn depth_image(size: UVec2, samples: vk::SampleCountFlags) -> Result<Self, vk::Result> {
        Self::new(
            size,
            DEPTH_FORMAT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            vk::ImageAspectFlags::DEPTH,
            samples,
            1,
            1,
        )
    }

    pub fn new(
        size: UVec2,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect_mask: vk::ImageAspectFlags,
        samples: vk::SampleCountFlags,
        level_count: u32,
        layer_count: u32,
    ) -> Result<VulkanImage, vk::Result> {
        let device = get_device();

        let (image, allocation) = {
            let image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .samples(samples)
                .extent(vk::Extent3D {
                    width: size.x,
                    height: size.y,
                    depth: 1,
                })
                .format(format)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(usage)
                .mip_levels(layer_count)
                .array_layers(level_count);

            let create_info = vma::AllocationCreateInfo {
                usage: vma::MemoryUsage::Auto,
                ..Default::default()
            };

            unsafe { device.allocator.create_image(&image_info, &create_info)? }
        };

        let image_view = {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .format(format)
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            unsafe { device.create_image_view(&create_info, None) }?
        };

        Ok(Self {
            image,
            image_view,
            allocation,
            format,
            layer_count,
            level_count,
            size,
            image_layout: vk::ImageLayout::UNDEFINED,
            aspect_mask,
        })
    }

    pub fn image_view_custom(
        &self,
        mut create_info: vk::ImageViewCreateInfo,
    ) -> Result<VulkanImageView, vk::Result> {
        create_info.image = self.image;
        VulkanImageView::new(&create_info)
    }

    pub fn image_view(&self) -> Result<VulkanImageView, vk::Result> {
        let create_info = vk::ImageViewCreateInfo::default()
            .format(self.format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: self.aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        self.image_view_custom(create_info)
    }

    pub unsafe fn image_barrier(
        &mut self,
        src_stage_mask: vk::PipelineStageFlags2,
        src_access_mask: vk::AccessFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
        dst_access_mask: vk::AccessFlags2,
        new_layout: vk::ImageLayout,
    ) -> vk::ImageMemoryBarrier2 {
        let barrier = vk::ImageMemoryBarrier2::default()
            .image(self.image)
            .src_stage_mask(src_stage_mask)
            .src_access_mask(src_access_mask)
            .dst_stage_mask(dst_stage_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(self.image_layout)
            .new_layout(new_layout)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: self.aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        self.image_layout = new_layout;

        barrier
    }

    pub fn destroy(&mut self) {
        if self.image.is_null() {
            return;
        }

        let device = get_device();

        unsafe {
            device
                .allocator
                .destroy_image(self.image, &mut self.allocation)
        }

        self.image = vk::Image::null();
    }
}

impl Drop for VulkanImage {
    fn drop(&mut self) {
        self.destroy();
    }
}

pub struct VulkanImageView {
    pub view: vk::ImageView,
}

impl VulkanImageView {
    pub fn new(create_info: &vk::ImageViewCreateInfo) -> Result<Self, vk::Result> {
        let device = get_device();

        let view = unsafe { device.create_image_view(create_info, None) }?;

        let view = Self { view };

        Ok(view)
    }

    pub fn destroy(&mut self) {
        if self.view.is_null() {
            return;
        }

        unsafe {
            get_device().destroy_image_view(self.view, None);
        }

        self.view = vk::ImageView::null();
    }
}

impl Deref for VulkanImageView {
    type Target = vk::ImageView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl Drop for VulkanImageView {
    fn drop(&mut self) {
        self.destroy()
    }
}
