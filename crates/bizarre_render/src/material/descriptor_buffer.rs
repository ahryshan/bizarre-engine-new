use core::slice;
use std::time::Duration;

use ash::vk;
use bizarre_log::core_trace;
use vma::Alloc;

use crate::{
    buffer::GpuBuffer,
    vulkan_context::{get_context, get_device, get_instance},
};

pub struct DescriptorBuffer {
    buffer: vk::Buffer,
    len: usize,
    allocation: vma::Allocation,
    element_size: vk::DeviceSize,
    element_offset: vk::DeviceSize,
    layout: vk::DescriptorSetLayout,
    additional_flags: vk::BufferUsageFlags,
}

type DeviceExt = ash::ext::descriptor_buffer::Device;

macro_rules! trace_sleep {
    ($($input:tt),+$(,)?) => {
        core_trace!($($input),+);
        std::thread::sleep(Duration::from_millis(10))
    };
}

impl DescriptorBuffer {
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

        let (element_size, element_offset) = unsafe {
            let size = device_ext.get_descriptor_set_layout_size(layout);
            let size = aligned_size(size, min_buffer_alignment);

            let offset = device_ext.get_descriptor_set_layout_binding_offset(layout, 0);

            (size, offset)
        };

        let (buffer, allocation) = unsafe {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(element_size * (len as vk::DeviceSize))
                .usage(
                    vk::BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT
                        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                        | additional_usage_flags,
                );
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
            element_size,
            additional_flags: additional_usage_flags,
        })
    }

    pub unsafe fn set_uniform_buffer(&mut self, buffer: &GpuBuffer, index: usize) {
        let device = get_device();

        let ad = device.get_buffer_address(*buffer.buffer());

        let addr_info = vk::DescriptorAddressInfoEXT::default()
            .address(ad)
            .range(buffer.size())
            .format(vk::Format::UNDEFINED);

        let descriptor_info = vk::DescriptorGetInfoEXT::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .data(vk::DescriptorDataEXT {
                p_uniform_buffer: &addr_info,
            });

        let descriptor = unsafe {
            let ptr = self.map_ptr::<u8>().unwrap();

            slice::from_raw_parts_mut(
                ptr.add(self.element_offset as usize + index * self.element_size as usize),
                self.element_size as usize,
            )
        };

        self.get_descriptor(&descriptor_info, descriptor);

        self.unmap_ptr()
    }

    pub fn get_descriptor(
        &self,
        descriptor_info: &vk::DescriptorGetInfoEXT,
        descriptor: &mut [u8],
    ) {
        let device_ext = device_ext();

        unsafe { device_ext.get_descriptor(descriptor_info, descriptor) }
    }

    pub fn binding_info(&self) -> vk::DescriptorBufferBindingInfoEXT {
        let device = get_device();
        let address = device.get_buffer_address(self.buffer);

        vk::DescriptorBufferBindingInfoEXT::default()
            .address(address)
            .usage(vk::BufferUsageFlags::RESOURCE_DESCRIPTOR_BUFFER_EXT | self.additional_flags)
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
