use std::{
    ffi::{c_char, CStr},
    ops::Deref,
};

use ash::vk::{self, PhysicalDeviceType};
use bizarre_log::{core_info, core_trace};
use thiserror::Error;

use crate::{instance::VulkanInstance, present_target::SwapchainSupportInfo};

const REQUIRED_EXTENSIONS: &'static [*const c_char] = &[ash::khr::swapchain::NAME.as_ptr()];

pub struct VulkanDevice {
    pub(crate) logical: ash::Device,
    pub(crate) physical: vk::PhysicalDevice,
    pub(crate) memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub(crate) queue_families: QueueFamilies,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) compute_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) cmd_pool: vk::CommandPool,
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error(transparent)]
    VulkanError(#[from] vk::Result),
    #[error(transparent)]
    CStrConvertFail(#[from] std::ffi::FromBytesUntilNulError),
    #[error("Could not find a suitable physical device")]
    NoSuitablePhysicalDevice,
    #[error("Could not find suitable memory")]
    NoSuitableMemory,
}

pub type DeviceResult<T> = Result<T, DeviceError>;

impl VulkanDevice {
    pub(crate) fn new(instance: &VulkanInstance) -> DeviceResult<Self> {
        let (physical, queue_families) =
            find_best_physical_device(instance).ok_or(DeviceError::NoSuitablePhysicalDevice)?;

        let name = get_pdevice_name(instance, physical);

        core_info!("Picked physical device: {name}");

        let queue_priorities = [1.0];

        let queue_create_infos = queue_families
            .unique_indices()
            .into_iter()
            .map(|idx| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(idx)
                    .flags(vk::DeviceQueueCreateFlags::empty())
                    .queue_priorities(&queue_priorities)
            })
            .collect::<Vec<_>>();

        let mut sync2 =
            vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&REQUIRED_EXTENSIONS)
            .push_next(&mut sync2);

        let logical = unsafe { instance.create_device(physical, &create_info, None)? };

        let graphics_queue = unsafe { logical.get_device_queue(queue_families.graphics, 0) };
        let compute_queue = unsafe { logical.get_device_queue(queue_families.compute, 0) };
        let present_queue = unsafe { logical.get_device_queue(queue_families.present, 0) };

        let cmd_pool = {
            let create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_families.graphics);

            unsafe { logical.create_command_pool(&create_info, None)? }
        };

        let memory_properties =
            { unsafe { instance.get_physical_device_memory_properties(physical) } };

        Ok(Self {
            physical,
            logical,
            queue_families,
            graphics_queue,
            compute_queue,
            present_queue,
            cmd_pool,
            memory_properties,
        })
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> DeviceResult<u32> {
        let iter = self.memory_properties.memory_types.into_iter().enumerate();

        for (i, mem_type) in iter {
            if type_filter & (1 << i) != 0 && mem_type.property_flags & properties == properties {
                return Ok(i as u32);
            }
        }

        Err(DeviceError::NoSuitableMemory)
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device_wait_idle();
            self.logical.destroy_command_pool(self.cmd_pool, None);
            self.logical.destroy_device(None);
        }
    }
}

impl Deref for VulkanDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.logical
    }
}

#[derive(Default, Debug, Clone)]
struct QueueFamiliesBuilder {
    graphics: Option<u32>,
    compute: Option<u32>,
    present: Option<u32>,
}

impl QueueFamiliesBuilder {
    fn try_build(self) -> Option<QueueFamilies> {
        if self.is_complete() {
            Some(QueueFamilies {
                graphics: self.graphics.unwrap(),
                compute: self.compute.unwrap(),
                present: self.present.unwrap(),
            })
        } else {
            None
        }
    }

    fn is_complete(&self) -> bool {
        self.compute.is_some() && self.graphics.is_some() && self.present.is_some()
    }
}

#[derive(Default, Debug, Clone)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub compute: u32,
    pub present: u32,
}

impl QueueFamilies {
    pub fn unique_indices(&self) -> Vec<u32> {
        let mut result = vec![self.graphics, self.compute];
        result.sort();
        result.dedup();
        result
    }
}

#[inline]
fn find_best_physical_device(
    instance: &VulkanInstance,
) -> Option<(vk::PhysicalDevice, QueueFamilies)> {
    let pdevices = unsafe { instance.enumerate_physical_devices() }.ok()?;

    let display = bizarre_window::get_wayland_display_ptr() as *mut vk::wl_display;
    let display = unsafe { &mut *display };

    let surface_loader =
        ash::khr::wayland_surface::Instance::new(&instance.entry, &instance.instance);

    let test_surface = {
        let wl_surface = bizarre_window::get_wayland_test_surface_ptr() as _;

        let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
            .display(display)
            .surface(wl_surface);

        unsafe { surface_loader.create_wayland_surface(&create_info, None) }.unwrap()
    };

    let mut rating = pdevices
        .iter()
        .map(|dev| rate_pdevice(instance, *dev, display, &surface_loader, test_surface))
        .filter_map(|rating| {
            if let Some((rate, ..)) = rating {
                if rate > 0 {
                    rating
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    unsafe {
        let surface_loader = ash::khr::surface::Instance::new(&instance.entry, &instance.instance);
        surface_loader.destroy_surface(test_surface, None);
    };

    if rating.is_empty() {
        return None;
    }

    rating.sort_by(|(a, ..), (b, ..)| a.cmp(b).reverse());

    let (_, best_device, queue_families) = rating.remove(0);

    Some((best_device, queue_families))
}

#[inline]
fn rate_pdevice(
    instance: &VulkanInstance,
    dev: vk::PhysicalDevice,
    display: &mut vk::wl_display,
    surface_loader: &ash::khr::wayland_surface::Instance,
    test_surface: vk::SurfaceKHR,
) -> Option<(u64, vk::PhysicalDevice, QueueFamilies)> {
    let props = unsafe { instance.get_physical_device_properties(dev) };
    let features = unsafe { instance.get_physical_device_features(dev) };

    if features.geometry_shader == 0 {
        return None;
    }

    let queue_families = if let Some(queue_famies) =
        find_queue_families(instance, dev, display, surface_loader).try_build()
    {
        queue_famies
    } else {
        return None;
    };

    let (support, swapchain_adequate) = swapchain_support(instance, dev, test_surface);

    if !check_pdevice_extensions(instance, dev) || !swapchain_adequate {
        return None;
    }

    let mut rating = 0;

    match props.device_type {
        PhysicalDeviceType::DISCRETE_GPU => rating += 100000,
        _ => return None,
    }

    rating += props.limits.max_image_dimension2_d as u64;

    Some((rating, dev, queue_families))
}

#[inline]
fn check_pdevice_extensions(instance: &VulkanInstance, dev: vk::PhysicalDevice) -> bool {
    let mut required = REQUIRED_EXTENSIONS
        .iter()
        .map(|name| unsafe { CStr::from_ptr(*name) })
        .collect::<Vec<_>>();

    unsafe { instance.enumerate_device_extension_properties(dev) }
        .unwrap()
        .into_iter()
        .for_each(|ext| {
            let name = if let Ok(name) = ext.extension_name_as_c_str() {
                name
            } else {
                return;
            };
            required.retain(|required| *required != name);
        });

    required.is_empty()
}

/// Returns SwapchainSupportInfo for a device and if it's adequate
#[inline]
fn swapchain_support(
    instance: &VulkanInstance,
    dev: vk::PhysicalDevice,
    test_surface: vk::SurfaceKHR,
) -> (SwapchainSupportInfo, bool) {
    let info = SwapchainSupportInfo::query_support_info(instance, dev, test_surface);

    let adequate = !info.formats.is_empty() && !info.present_modes.is_empty();

    (info, adequate)
}

#[inline]
fn get_pdevice_name(instance: &VulkanInstance, dev: vk::PhysicalDevice) -> String {
    unsafe {
        instance
            .get_physical_device_properties(dev)
            .device_name_as_c_str()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

#[inline]
fn find_queue_families(
    instance: &VulkanInstance,
    dev: vk::PhysicalDevice,
    display: &mut vk::wl_display,
    surface_loader: &ash::khr::wayland_surface::Instance,
) -> QueueFamiliesBuilder {
    let families = unsafe { instance.get_physical_device_queue_family_properties(dev) };

    let mut result = QueueFamiliesBuilder::default();

    for (i, family) in families.into_iter().enumerate() {
        if family.queue_flags.intersects(vk::QueueFlags::GRAPHICS) {
            result.graphics = Some(i as u32);
        }
        if family.queue_flags.intersects(vk::QueueFlags::COMPUTE) {
            result.compute = Some(i as u32);
        }

        if query_present_support(surface_loader, display, dev, i as u32) {
            result.present = Some(i as u32);
        }

        if result.is_complete() {
            break;
        }
    }

    result
}

fn query_present_support(
    surface_loader: &ash::khr::wayland_surface::Instance,
    display: &mut vk::wl_display,
    dev: vk::PhysicalDevice,
    queue_index: u32,
) -> bool {
    unsafe {
        surface_loader.get_physical_device_wayland_presentation_support(dev, queue_index, display)
    }
}
