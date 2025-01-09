use std::sync::{LazyLock, OnceLock};

use ash::vk::{
    self, native::StdVideoAV1TransferCharacteristics_STD_VIDEO_AV1_TRANSFER_CHARACTERISTICS_INVALID,
};
use bizarre_log::core_fatal;

use crate::{device::LogicalDevice, instance::VulkanInstance};

static CTX: LazyLock<VulkanContext> = LazyLock::new(|| match VulkanContext::new() {
    Ok(ctx) => ctx,
    Err(err) => {
        let message = format!("Failed to init Vulkan context {err:?}");
        core_fatal!("{}", message);
        panic!("{}", message);
    }
});

pub struct VulkanContext {
    device: LogicalDevice,
    instance: VulkanInstance,
}

impl VulkanContext {
    pub fn new() -> Result<Self, vk::Result> {
        let instance = VulkanInstance::new();
        let device = LogicalDevice::new(&instance).unwrap();

        Ok(Self { device, instance })
    }

    pub fn device(&self) -> &LogicalDevice {
        &self.device
    }

    pub fn instance(&self) -> &VulkanInstance {
        &self.instance
    }
}

pub fn get_context() -> &'static VulkanContext {
    &CTX
}

pub fn get_device() -> &'static LogicalDevice {
    &CTX.device
}

pub fn get_instance() -> &'static VulkanInstance {
    &CTX.instance
}
