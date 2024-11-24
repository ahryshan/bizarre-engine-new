use std::ops::{Deref, DerefMut};

use ash::vk;
use bizarre_log::core_trace;
use thiserror::Error;

use crate::{
    debug_messenger::{populate_debug_messenger_create_info, DebugMessenger},
    device::{DeviceResult, VulkanDevice},
};

#[derive(Error, Debug)]
pub enum InstanceError {
    #[error("Failed to create a Vulkan instance: {0}")]
    CreationError(#[from] vk::Result),
}

pub type InstanceResult<T> = Result<T, InstanceError>;

pub struct VulkanInstance {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) debug_messenger: Option<DebugMessenger>,
}

impl VulkanInstance {
    pub fn new() -> Self {
        let entry = ash::Entry::linked();

        let instance = 'create_instance: {
            unsafe {
                let mut extentions = PLATFORM_EXTENTIONS.to_vec();
                extentions.extend_from_slice(ADDITIONAL_EXTENTIONS);

                let application_info =
                    vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

                let create_info = vk::InstanceCreateInfo::default()
                    .application_info(&application_info)
                    .enabled_extension_names(&extentions)
                    .enabled_layer_names(&LAYERS);

                #[cfg(debug_assertions)]
                {
                    let mut debug_utils = vk::DebugUtilsMessengerCreateInfoEXT::default();

                    populate_debug_messenger_create_info(&mut debug_utils);

                    let create_info = create_info.push_next(&mut debug_utils);

                    break 'create_instance entry.create_instance(&create_info, None).unwrap();
                }

                entry.create_instance(&create_info, None).unwrap()
            }
        };

        #[cfg(debug_assertions)]
        let debug_messenger = Some(DebugMessenger::new(&entry, &instance));

        #[cfg(not(debug_assertions))]
        let debug_messenger = None;

        Self {
            entry,
            instance,
            debug_messenger,
        }
    }

    pub fn create_device_ext(&self) -> DeviceResult<VulkanDevice> {
        VulkanDevice::new(self)
    }

    pub fn entry_ext(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn inner(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Deref for VulkanInstance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for VulkanInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        let messenger = self.debug_messenger.take();
        drop(messenger);
        unsafe { self.instance.destroy_instance(None) };
    }
}

#[cfg(target_os = "linux")]
const PLATFORM_EXTENTIONS: &'static [*const std::ffi::c_char] = &[
    vk::KHR_SURFACE_NAME.as_ptr(),
    vk::KHR_WAYLAND_SURFACE_NAME.as_ptr(),
];

#[cfg(debug_assertions)]
const ADDITIONAL_EXTENTIONS: &'static [*const std::ffi::c_char] =
    &[vk::EXT_DEBUG_UTILS_NAME.as_ptr()];

#[cfg(debug_assertions)]
const LAYERS: &'static [*const std::ffi::c_char] = unsafe {
    &[std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0").as_ptr()]
};

#[cfg(not(debug_assertions))]
const LAYERS: &'static [*const std::ffi::c_char] = &[];
