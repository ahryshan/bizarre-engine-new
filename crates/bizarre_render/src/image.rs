use ash::vk;
use nalgebra_glm::UVec2;

use crate::device::VulkanDevice;

pub struct VulkanImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub size: UVec2,
}

impl VulkanImage {
    pub fn new(device: &VulkanDevice, size: UVec2) -> Result<VulkanImage, vk::Result> {
        let image = {
            let create_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .samples(vk::SampleCountFlags::TYPE_1)
                .extent(vk::Extent3D {
                    width: size.x,
                    height: size.y,
                    depth: 1,
                })
                .format(vk::Format::R8G8B8A8_SRGB)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                .mip_levels(1)
                .array_layers(1);

            unsafe { device.create_image(&create_info, None)? }
        };

        let memory = {
            let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

            let allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(
                    device
                        .find_memory_type(
                            mem_requirements.memory_type_bits,
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        )
                        .unwrap(),
                );

            unsafe { device.allocate_memory(&allocate_info, None) }?
        };

        unsafe {
            device.bind_image_memory(image, memory, 0)?;
        }

        let image_view = {
            let create_info = vk::ImageViewCreateInfo::default()
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .format(vk::Format::R8G8B8A8_SRGB)
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            unsafe { device.create_image_view(&create_info, None)? }
        };

        Ok(Self {
            image,
            image_view,
            memory,
            size,
        })
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}