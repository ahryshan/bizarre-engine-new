use ash::vk;

use crate::device::VulkanDevice;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: usize,
}

impl Buffer {
    pub fn new(
        device: &VulkanDevice,
        size: usize,
        usage: vk::BufferUsageFlags,
        flags: vk::MemoryPropertyFlags,
    ) -> Result<Self, vk::Result> {
        let buffer = {
            let create_info = vk::BufferCreateInfo::default();
            unsafe { device.create_buffer(&create_info, None)? }
        };

        let mem_index = {
            let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
            device
                .find_memory_type(mem_requirements.memory_type_bits, flags)
                .unwrap();
        };

        let memory = {
            let allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(size as vk::DeviceSize)
                .memory_type_index(mem_index);
        };

        todo!()
    }
}
