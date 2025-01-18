use core::slice;
use std::time::Duration;

use ash::vk;
use bizarre_log::core_trace;
use vma::Alloc;

use crate::{
    buffer::GpuBuffer,
    image::VulkanImage,
    vulkan_context::{get_context, get_device, get_instance},
};

pub struct DescriptorBuffer {
    buffer: vk::Buffer,
    len: usize,
    allocation: vma::Allocation,
    element_stride: vk::DeviceSize,
    element_offset: vk::DeviceSize,
    layout: vk::DescriptorSetLayout,
    usage_flags: vk::BufferUsageFlags,
}

type DeviceExt = ash::ext::descriptor_buffer::Device;

macro_rules! trace_sleep {
    ($($input:tt),+$(,)?) => {
        core_trace!($($input),+);
        std::thread::sleep(Duration::from_millis(10))
    };
}

impl DescriptorBuffer {
    pub fn uniform_buffers(len: usize) -> Result<Self, vk::Result> {
        Self::new(
            len,
            &[vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                ..Default::default()
            }],
            vk::BufferUsageFlags::empty(),
        )
    }

    pub fn textures(len: usize) -> Result<Self, vk::Result> {
        Self::new(
            len,
            &[vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                ..Default::default()
            }],
            vk::BufferUsageFlags::SAMPLER_DESCRIPTOR_BUFFER_EXT,
        )
    }

    pub fn new(
        len: usize,
        bindings: &[vk::DescriptorSetLayoutBinding],
        additional_usage_flags: vk::BufferUsageFlags,
    ) -> Result<Self, vk::Result> {
        let device = get_device();

        let layout = unsafe {
            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(bindings)
                .flags(vk::DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT);

            device.create_descriptor_set_layout(&create_info, None)?
        };

        let device_ext = device_ext();

        let min_buffer_alignment = device
            .physical
            .descriptor_buffer_props
            .descriptor_buffer_offset_alignment;

        let (element_stride, element_offset) = unsafe {
            let size = device_ext.get_descriptor_set_layout_size(layout);
            let size = aligned_size(size, min_buffer_alignment);

            let offset = device_ext.get_descriptor_set_layout_binding_offset(layout, 0);

            (size, offset)
        };

        let usage_flags = vk::BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | additional_usage_flags;

        let (buffer, allocation) = unsafe {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(element_stride * (len as vk::DeviceSize))
                .usage(usage_flags);
            let create_info = vma::AllocationCreateInfo {
                usage: vma::MemoryUsage::Auto,
                flags: vma::AllocationCreateFlags::HOST_ACCESS_RANDOM,
                ..Default::default()
            };

            device.allocator.create_buffer_with_alignment(
                &buffer_info,
                &create_info,
                min_buffer_alignment,
            )?
        };

        Ok(Self {
            len,
            layout,
            buffer,
            allocation,
            element_offset,
            element_stride,
            usage_flags,
        })
    }

    pub fn element_stride(&self) -> vk::DeviceSize {
        self.element_stride
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub unsafe fn set_uniform_buffer_unchecked(
        &mut self,
        buffer: &GpuBuffer,
        index: usize,
    ) -> vk::DeviceSize {
        let device = get_device();

        let ad = device.get_buffer_address(buffer.buffer());

        let addr_info = vk::DescriptorAddressInfoEXT::default()
            .address(ad)
            .range(buffer.size())
            .format(vk::Format::UNDEFINED);

        let descriptor_info = vk::DescriptorGetInfoEXT::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .data(vk::DescriptorDataEXT {
                p_uniform_buffer: &addr_info,
            });

        let offset = self.element_offset as usize + index * self.element_stride as usize;

        let descriptor = unsafe {
            let ptr = self.map_ptr::<u8>().unwrap();

            slice::from_raw_parts_mut(
                ptr.add(offset),
                device
                    .physical
                    .descriptor_buffer_props
                    .uniform_buffer_descriptor_size,
            )
        };

        self.get_descriptor(&descriptor_info, descriptor);

        self.unmap_ptr();

        offset as vk::DeviceSize
    }

    pub unsafe fn set_input_attachment_unchecked(
        &mut self,
        texture: &VulkanImage,
        index: usize,
    ) -> vk::DeviceSize {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(texture.image_layout)
            .image_view(texture.image_view);

        let descriptor_info = vk::DescriptorGetInfoEXT::default()
            .ty(vk::DescriptorType::INPUT_ATTACHMENT)
            .data(vk::DescriptorDataEXT {
                p_input_attachment_image: &image_info,
            });

        let offset = self.element_offset as usize + index * self.element_stride as usize;

        let descriptor = {
            let ptr = self.map_ptr::<u8>().unwrap();

            slice::from_raw_parts_mut(
                ptr.add(offset),
                get_device()
                    .physical
                    .descriptor_buffer_props
                    .input_attachment_descriptor_size,
            )
        };

        self.get_descriptor(&descriptor_info, descriptor);

        self.unmap_ptr();

        offset as vk::DeviceSize
    }

    pub unsafe fn set_texture_unchecked(
        &mut self,
        texture: &VulkanImage,
        sampler: vk::Sampler,
        index: usize,
    ) -> vk::DeviceSize {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(texture.image_layout)
            .image_view(texture.image_view)
            .sampler(sampler);

        let descriptor_info = vk::DescriptorGetInfoEXT::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .data(vk::DescriptorDataEXT {
                p_combined_image_sampler: &image_info,
            });

        let offset = self.element_offset as usize + index * self.element_stride as usize;

        let descriptor = {
            let ptr = self.map_ptr::<u8>().unwrap();

            slice::from_raw_parts_mut(ptr.add(offset), self.element_stride as usize)
        };

        self.get_descriptor(&descriptor_info, descriptor);

        self.unmap_ptr();

        offset as vk::DeviceSize
    }

    pub fn get_descriptor(
        &self,
        descriptor_info: &vk::DescriptorGetInfoEXT,
        descriptor: &mut [u8],
    ) {
        let device_ext = device_ext();

        unsafe { device_ext.get_descriptor(descriptor_info, descriptor) }
    }

    pub fn binding_info(&self) -> vk::DescriptorBufferBindingInfoEXT<'static> {
        let device = get_device();
        let address = device.get_buffer_address(self.buffer);

        vk::DescriptorBufferBindingInfoEXT::default()
            .address(address)
            .usage(self.usage_flags)
    }

    pub unsafe fn map_ptr<T>(&mut self) -> Result<*mut T, vk::Result> {
        let device = get_context().device();
        device
            .allocator
            .map_memory(&mut self.allocation)
            .map(|ptr| ptr as *mut T)
    }

    pub unsafe fn unmap_ptr(&mut self) {
        let device = get_context().device();
        device.allocator.unmap_memory(&mut self.allocation);
    }
}

impl Drop for DescriptorBuffer {
    fn drop(&mut self) {
        unsafe {
            get_device()
                .allocator
                .destroy_buffer(self.buffer, &mut self.allocation);
            get_device().destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

fn aligned_size(value: vk::DeviceSize, alignment: vk::DeviceSize) -> vk::DeviceSize {
    (value + alignment - 1) & !(alignment - 1)
}

pub(crate) fn device_ext() -> DeviceExt {
    DeviceExt::new(get_instance(), get_device())
}
