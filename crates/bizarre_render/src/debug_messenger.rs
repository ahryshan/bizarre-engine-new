use std::ffi::c_void;

use ash::vk;
use bizarre_log::{log, LogLevel};

pub struct DebugMessenger {
    loader: ash::ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub(crate) fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let loader = ash::ext::debug_utils::Instance::new(entry, instance);

        let mut create_info = vk::DebugUtilsMessengerCreateInfoEXT::default();
        populate_debug_messenger_create_info(&mut create_info);

        let messenger = unsafe {
            loader
                .create_debug_utils_messenger(&create_info, None)
                .unwrap()
        };

        Self { loader, messenger }
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        }
    }
}

pub fn populate_debug_messenger_create_info<'a>(
    create_info: &'a mut vk::DebugUtilsMessengerCreateInfoEXT,
) {
    type Severity = vk::DebugUtilsMessageSeverityFlagsEXT;
    type Type = vk::DebugUtilsMessageTypeFlagsEXT;

    create_info.pfn_user_callback = Some(messenger_callback);
    create_info.message_severity = Severity::WARNING | Severity::ERROR;
    create_info.message_type = Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION;
}

unsafe extern "system" fn messenger_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> u32 {
    const LOGGER_NAME: &'static str = "engine";
    let level = match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => LogLevel::Trace,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => LogLevel::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => LogLevel::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => LogLevel::Error,
        _ => unreachable!(),
    };
    let msg = unsafe {
        (*callback_data)
            .message_as_c_str()
            .map(|msg| msg.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    log!(LOGGER_NAME, level, "Vulkan validation: {msg}");
    0
}
