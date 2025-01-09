use ash::vk;
use bizarre_core::Handle;

use crate::{
    device::LogicalDevice,
    material::{
        material_binding::{BindingType, MaterialBinding},
        MaterialError,
    },
    vulkan_context::get_device,
};

use super::{
    descriptor_buffer::DescriptorBuffer,
    material_binding::{BindObject, BindObjectSet},
    Material, MaterialHandle, MaterialResult,
};

pub type MaterialInstanceHandle = Handle<MaterialInstance>;

pub struct MaterialInstance {
    material: MaterialHandle,
    bindings: BindObjectSet,
    uniforms: Option<DescriptorBuffer>,
    textures: Option<DescriptorBuffer>,
    set_updated: bool,
}

impl MaterialInstance {
    pub fn new(material_handle: MaterialHandle, material: &Material) -> MaterialResult<Self> {
        let bindings = BindObjectSet::from(&material.bindings);

        let uniforms = {
            let uniform_bindings = material
                .bindings
                .iter()
                .filter_map(|b| match b.binding_type {
                    BindingType::StorageBuffer | BindingType::UniformBuffer => {
                        Some(vk::DescriptorSetLayoutBinding::from(b))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            if uniform_bindings.len() > 0 {
                let buffer =
                    DescriptorBuffer::new(1, &uniform_bindings, vk::BufferUsageFlags::empty())?;
                Some(buffer)
            } else {
                None
            }
        };

        let textures = {
            let texture_bindings = material
                .bindings
                .iter()
                .filter_map(|b| match b.binding_type {
                    BindingType::InputAttachment | BindingType::Texture => {
                        Some(vk::DescriptorSetLayoutBinding::from(b))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            if texture_bindings.len() > 0 {
                let buffer = DescriptorBuffer::new(
                    1,
                    &texture_bindings,
                    vk::BufferUsageFlags::SAMPLER_DESCRIPTOR_BUFFER_EXT,
                )?;
                Some(buffer)
            } else {
                None
            }
        };

        let ret = Self {
            material: material_handle,
            bindings,
            uniforms,
            textures,
            set_updated: false,
        };

        Ok(ret)
    }

    pub fn bind(
        &mut self,
        device: &LogicalDevice,
        cmd: vk::CommandBuffer,
        material: &Material,
    ) -> MaterialResult<()> {
        todo!();
    }

    pub fn update_binding(&mut self, binding: usize, object: BindObject) -> MaterialResult<()> {
        todo!();
    }

    fn write_descriptor_buffer(&self, device: &LogicalDevice) {
        todo!();
    }
}
