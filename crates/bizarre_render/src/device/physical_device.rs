use std::ops::Deref;

use ash::vk;

use crate::instance::VulkanInstance;

pub struct PhysicalDevice {
    pub device: vk::PhysicalDevice,
    pub device_props: vk::PhysicalDeviceProperties,
    pub descriptor_buffer_props: vk::PhysicalDeviceDescriptorBufferPropertiesEXT<'static>,
}

impl PhysicalDevice {
    pub fn new(instance: &VulkanInstance, device: vk::PhysicalDevice) -> Self {
        let mut descriptor_props = vk::PhysicalDeviceDescriptorBufferPropertiesEXT::default();

        let (device_props, descriptor_props) = unsafe {
            let mut device_props =
                vk::PhysicalDeviceProperties2::default().push_next(&mut descriptor_props);

            instance.get_physical_device_properties2(device, &mut device_props);

            (device_props.properties, descriptor_props)
        };

        Self {
            device,
            device_props,
            descriptor_buffer_props: descriptor_props,
        }
    }
}

impl Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
