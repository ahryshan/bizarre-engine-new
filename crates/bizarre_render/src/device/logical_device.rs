use std::{
    ffi::{c_char, CStr},
    ops::Deref,
};

use ash::{
    ext::memory_priority,
    vk::{self, PhysicalDeviceType},
};
use bizarre_log::{core_info, core_trace};
use thiserror::Error;

use crate::{instance::VulkanInstance, present_target::SwapchainSupportInfo};

use super::PhysicalDevice;

const REQUIRED_EXTENSIONS: &'static [*const c_char] = &[
    ash::khr::swapchain::NAME.as_ptr(),
    ash::ext::descriptor_buffer::NAME.as_ptr(),
];

pub struct LogicalDevice {
    pub(crate) logical: ash::Device,
    pub(crate) physical: PhysicalDevice,
    pub(crate) queue_families: QueueFamilies,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) compute_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) cmd_pool: vk::CommandPool,
    pub(crate) descriptor_pool: vk::DescriptorPool,
    pub(crate) allocator: vma::Allocator,
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

impl LogicalDevice {
    pub(crate) fn new(instance: &VulkanInstance) -> DeviceResult<Self> {
        let (physical, queue_families) =
            find_best_physical_device(instance).ok_or(DeviceError::NoSuitablePhysicalDevice)?;

        let physical = PhysicalDevice::new(instance, physical);

        let name = get_pdevice_name(instance, *physical);

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

        let mut dynamic_rendering =
            vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

        let mut buffer_device_address =
            vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);

        let mut descriptor_buffer =
            vk::PhysicalDeviceDescriptorBufferFeaturesEXT::default().descriptor_buffer(true);

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&REQUIRED_EXTENSIONS)
            .push_next(&mut sync2)
            .push_next(&mut dynamic_rendering)
            .push_next(&mut buffer_device_address)
            .push_next(&mut descriptor_buffer);

        let logical = unsafe { instance.create_device(*physical, &create_info, None)? };

        let graphics_queue = unsafe { logical.get_device_queue(queue_families.graphics, 0) };
        let compute_queue = unsafe { logical.get_device_queue(queue_families.compute, 0) };
        let present_queue = unsafe { logical.get_device_queue(queue_families.present, 0) };

        let cmd_pool = {
            let create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_families.graphics);

            unsafe { logical.create_command_pool(&create_info, None)? }
        };

        let allocator = {
            let create_flags =
                unsafe { instance.enumerate_device_extension_properties(*physical) }?
                    .into_iter()
                    .filter_map(|ext| {
                        VMA_OPT_EXTENSIONS.into_iter().find_map(|vma_ext| {
                            if vma_ext.name == ext.extension_name_as_c_str().ok()? {
                                Some(vma::AllocatorCreateFlags::from_bits(vma_ext.flag.bits())?)
                            } else {
                                None
                            }
                        })
                    })
                    .fold(vma::AllocatorCreateFlags::empty(), |acc, curr| acc | curr);

            let mut create_info = vma::AllocatorCreateInfo::new(&instance, &logical, *physical);
            create_info.flags = create_flags;

            unsafe { vma::Allocator::new(create_info) }?
        };

        let descriptor_pool = unsafe {
            let pool_sizes = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: 32,
            }];

            let create_info = vk::DescriptorPoolCreateInfo::default()
                .max_sets(256)
                .pool_sizes(&pool_sizes);

            logical.create_descriptor_pool(&create_info, None)?
        };

        Ok(Self {
            physical,
            logical,
            queue_families,
            graphics_queue,
            compute_queue,
            present_queue,
            cmd_pool,
            descriptor_pool,
            allocator,
        })
    }

    pub(crate) fn get_buffer_address(&self, buffer: vk::Buffer) -> vk::DeviceAddress {
        let addr_info = vk::BufferDeviceAddressInfo::default().buffer(buffer);
        unsafe { self.get_buffer_device_address(&addr_info) }
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.device_wait_idle();
            self.logical.destroy_command_pool(self.cmd_pool, None);
            self.logical
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.logical.destroy_device(None);
        }
    }
}

impl Deref for LogicalDevice {
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

struct VmaExtension {
    name: &'static CStr,
    flag: vma::AllocatorCreateFlags,
}

const VMA_OPT_EXTENSIONS: &[VmaExtension] = &[
    VmaExtension {
        name: ash::khr::dedicated_allocation::NAME,
        flag: vma::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION,
    },
    VmaExtension {
        name: ash::khr::bind_memory2::NAME,
        flag: vma::AllocatorCreateFlags::KHR_BIND_MEMORY2,
    },
    VmaExtension {
        name: ash::khr::maintenance4::NAME,
        flag: vma::AllocatorCreateFlags::KHR_MAINTENANCE4,
    },
    VmaExtension {
        name: ash::khr::maintenance5::NAME,
        flag: vma::AllocatorCreateFlags::KHR_MAINTENANCE5,
    },
    VmaExtension {
        name: ash::ext::memory_budget::NAME,
        flag: vma::AllocatorCreateFlags::EXT_MEMORY_BUDGET,
    },
    VmaExtension {
        name: ash::ext::memory_priority::NAME,
        flag: vma::AllocatorCreateFlags::EXT_MEMORY_PRIORITY,
    },
    VmaExtension {
        name: ash::khr::buffer_device_address::NAME,
        flag: vma::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS,
    },
    VmaExtension {
        name: ash::amd::device_coherent_memory::NAME,
        flag: vma::AllocatorCreateFlags::AMD_DEVICE_COHERENT_MEMORY,
    },
];
