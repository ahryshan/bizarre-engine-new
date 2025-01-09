use std::ffi::c_void;

use ash::{nv::shader_subgroup_partitioned, vk};
use bizarre_core::Handle;
use bizarre_log::{core_info, core_trace, core_warn};
use nalgebra_glm::UVec2;
use thiserror::Error;

use crate::{
    device::LogicalDevice,
    image::VulkanImage,
    instance::VulkanInstance,
    render_target::{ImageRenderTarget, RenderData},
};

pub type PresentTargetHandle = Handle<PresentTarget>;

#[derive(Error, Debug)]
pub enum PresentError {
    #[error("Invalid render target")]
    InvalidRenderTarget,
    #[error("Invalid present target")]
    InvalidPresentTarget,
    #[error(transparent)]
    VulkanError(#[from] vk::Result),
}

pub type PresentResult<T> = Result<T, PresentError>;

#[derive(Clone, Debug)]
pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct PresentData {
    pub cmd_buffer: Option<vk::CommandBuffer>,
    pub swapchain: vk::SwapchainKHR,
    pub image_acquired: vk::Semaphore,
    pub image_ready: vk::Semaphore,
    pub image_index: u32,
}

impl SwapchainSupportInfo {
    pub(crate) fn query_support_info(
        instance: &VulkanInstance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Self {
        let surface_loader = ash::khr::surface::Instance::new(&instance.entry, &instance.instance);

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(device, surface)
                .unwrap()
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(device, surface)
                .unwrap()
        };

        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(device, surface)
                .unwrap()
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub struct PresentTarget {
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    size: UVec2,
    surface_format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    present_cmd_buffers: Vec<vk::CommandBuffer>,
    image_acquired: Vec<vk::Semaphore>,
    image_ready: Vec<vk::Semaphore>,

    next_image_index: u32,
}

impl PresentTarget {
    pub(crate) unsafe fn new(
        instance: &VulkanInstance,
        device: &LogicalDevice,
        cmd_pool: vk::CommandPool,
        extent: UVec2,
        display: *mut vk::wl_display,
        surface: *mut c_void,
    ) -> Result<Self, vk::Result> {
        let wl_surface_loader =
            ash::khr::wayland_surface::Instance::new(&instance.entry, &instance.instance);

        let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
            .display(display)
            .surface(surface);

        let surface = wl_surface_loader.create_wayland_surface(&create_info, None)?;

        let swapchain_loader =
            ash::khr::swapchain::Device::new(&instance.instance, &device.logical);

        let support = SwapchainSupportInfo::query_support_info(instance, *device.physical, surface);

        let present_mode = choose_present_mode(&support.present_modes);
        let format = choose_surface_format(&support.formats);

        let extent = {
            if support.capabilities.current_extent.width != u32::MAX {
                support.capabilities.current_extent
            } else {
                vk::Extent2D {
                    width: extent.x,
                    height: extent.y,
                }
            }
        };

        let image_count = if support.capabilities.max_image_count > 0 {
            (support.capabilities.min_image_count + 1).min(support.capabilities.max_image_count)
        } else {
            support.capabilities.min_image_count + 1
        };

        let (swapchain, images, image_views) = create_swapchain(
            device,
            &swapchain_loader,
            extent,
            image_count,
            present_mode,
            *format,
            surface,
            None,
        )
        .unwrap();

        let surface_loader = ash::khr::surface::Instance::new(&instance.entry, &instance.instance);

        let present_cmd_buffers = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(cmd_pool)
                .command_buffer_count(images.len() as u32)
                .level(vk::CommandBufferLevel::PRIMARY);

            device.allocate_command_buffers(&allocate_info)?
        };

        let image_acquired = images
            .iter()
            .map(|_| {
                let create_info = vk::SemaphoreCreateInfo::default();
                device.create_semaphore(&create_info, None)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let image_ready = images
            .iter()
            .map(|_| {
                let create_info = vk::SemaphoreCreateInfo::default();
                device.create_semaphore(&create_info, None)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let present_target = Self {
            surface_loader,
            surface,
            swapchain_loader,
            swapchain,
            surface_format: *format,
            present_mode,
            images,
            size: UVec2::new(extent.width, extent.height),
            image_views,
            present_cmd_buffers,
            image_acquired,
            image_ready,

            next_image_index: 0,
        };

        Ok(present_target)
    }

    pub fn image_count(&self) -> u32 {
        self.images.len() as u32
    }

    pub fn record_present(
        &mut self,
        device: &LogicalDevice,
        render_image: &VulkanImage,
    ) -> PresentResult<PresentData> {
        let image_acquired = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&create_info, None)? }
        };

        let (image_index, suboptimal) = unsafe {
            self.swapchain_loader
                .acquire_next_image(self.swapchain, u64::MAX, image_acquired, vk::Fence::null())
                .unwrap()
        };

        if suboptimal {
            core_warn!("Swapchain is suboptimal!");
        }

        let old_semaphore = &mut self.image_acquired[image_index as usize];

        unsafe {
            device.destroy_semaphore(*old_semaphore, None);
            *old_semaphore = image_acquired;
        }

        let cmd = self.present_cmd_buffers[image_index as usize];
        let image = self.images[image_index as usize];
        let image_ready = self.image_ready[image_index as usize];

        self.record_present_cmd(device, cmd, image, render_image)?;

        Ok(PresentData {
            cmd_buffer: Some(cmd),
            swapchain: self.swapchain,
            image_acquired,
            image_ready,
            image_index,
        })
    }

    fn record_present_cmd(
        &self,
        device: &LogicalDevice,
        cmd: vk::CommandBuffer,
        present_image: vk::Image,
        render_image: &VulkanImage,
    ) -> PresentResult<()> {
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe { device.begin_command_buffer(cmd, &begin_info) }?;

        unsafe {
            let to_transfer_barriers = [vk::ImageMemoryBarrier2::default()
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(present_image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1)
                        .base_array_layer(0)
                        .level_count(1)
                        .base_mip_level(0),
                )
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)];

            let dependency_info =
                vk::DependencyInfo::default().image_memory_barriers(&to_transfer_barriers);

            device.cmd_pipeline_barrier2(cmd, &dependency_info);

            let subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1)
                .mip_level(0);

            let src_offsets = [
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: render_image.size.x as i32,
                    y: render_image.size.y as i32,
                    z: 1,
                },
            ];

            let dst_offsets = [
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: self.size.x as i32,
                    y: self.size.y as i32,
                    z: 1,
                },
            ];
            let regions = [vk::ImageBlit2::default()
                .src_offsets(src_offsets.clone())
                .dst_offsets(dst_offsets.clone())
                .src_subresource(subresource)
                .dst_subresource(subresource)];

            let blit_info = vk::BlitImageInfo2::default()
                .src_image(render_image.image)
                .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .dst_image(present_image)
                .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .filter(vk::Filter::NEAREST)
                .regions(&regions);

            device.cmd_blit_image2(cmd, &blit_info);

            let to_present_barrier = [vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(present_image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1)
                        .base_array_layer(0)
                        .level_count(1)
                        .base_mip_level(0),
                )
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

            let dependency_info =
                vk::DependencyInfo::default().image_memory_barriers(&to_present_barrier);

            device.cmd_pipeline_barrier2(cmd, &dependency_info);

            device.end_command_buffer(cmd)?;
        }

        Ok(())
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn resize(&mut self, device: &LogicalDevice, size: UVec2) -> PresentResult<()> {
        let extent = vk::Extent2D {
            width: size.x,
            height: size.y,
        };
        let (swapchain, images, image_views) = create_swapchain(
            device,
            &self.swapchain_loader,
            extent,
            self.images.len() as u32,
            self.present_mode,
            self.surface_format,
            self.surface,
            Some(self.swapchain),
        )?;

        unsafe {
            self.image_views
                .drain(..)
                .for_each(|view| device.destroy_image_view(view, None));

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }

        self.swapchain = swapchain;
        self.images = images;
        self.image_views = image_views;

        self.size = size;

        Ok(())
    }

    pub fn destroy(&mut self, device: &LogicalDevice) {
        self.image_views
            .drain(..)
            .for_each(|image_view| unsafe { device.logical.destroy_image_view(image_view, None) });

        self.image_acquired
            .drain(..)
            .for_each(|semaphore| unsafe { device.destroy_semaphore(semaphore, None) });

        self.image_ready
            .drain(..)
            .for_each(|semaphore| unsafe { device.destroy_semaphore(semaphore, None) });

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        }
        unsafe { self.surface_loader.destroy_surface(self.surface, None) }
    }
}

#[inline]
fn choose_surface_format(formats: &Vec<vk::SurfaceFormatKHR>) -> &vk::SurfaceFormatKHR {
    for format in formats {
        if format.format == vk::Format::R8G8B8A8_UNORM
            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return format;
        }
    }

    &formats[0]
}

#[inline]
fn choose_present_mode(modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for mode in modes {
        if mode == &vk::PresentModeKHR::MAILBOX {
            return *mode;
        }
    }

    vk::PresentModeKHR::FIFO
}

#[inline]
fn create_swapchain(
    device: &LogicalDevice,
    swapchain_loader: &ash::khr::swapchain::Device,
    extent: vk::Extent2D,
    image_count: u32,
    present_mode: vk::PresentModeKHR,
    format: vk::SurfaceFormatKHR,
    surface: vk::SurfaceKHR,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, Vec<vk::ImageView>), vk::Result> {
    let create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .image_array_layers(1)
        .image_color_space(format.color_space)
        .image_format(format.format)
        .image_extent(extent)
        .present_mode(present_mode)
        .min_image_count(image_count)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
        .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
        .clipped(true)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE);

    let queue_family_indices = [
        device.queue_families.graphics,
        device.queue_families.present,
    ];

    let create_info = if device.queue_families.graphics != device.queue_families.present {
        create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_family_indices)
    } else {
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };

    let create_info = if let Some(old_swapchain) = old_swapchain {
        create_info.old_swapchain(old_swapchain)
    } else {
        create_info
    };

    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;

    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

    let image_views = images
        .iter()
        .map(|image| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .components(
                    vk::ComponentMapping::default()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY),
                )
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            unsafe { device.logical.create_image_view(&create_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((swapchain, images, image_views))
}
