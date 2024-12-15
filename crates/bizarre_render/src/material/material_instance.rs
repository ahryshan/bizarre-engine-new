use ash::vk;
use bizarre_core::Handle;

use crate::{
    device::VulkanDevice,
    material::{material_binding::BindingType, MaterialError},
};

use super::{
    material_binding::{BindObject, BindObjectSet},
    Material, MaterialHandle, MaterialResult,
};

pub type MaterialInstanceHandle = Handle<MaterialInstance>;

pub struct MaterialInstance {
    material: MaterialHandle,
    bindings: BindObjectSet,
    descriptor_set: vk::DescriptorSet,
    set_updated: bool,
}

impl MaterialInstance {
    pub fn bind(
        &mut self,
        device: &VulkanDevice,
        cmd: vk::CommandBuffer,
        material: &Material,
    ) -> MaterialResult<()> {
        let has_unbound = self.bindings.iter().any(|b| match b {
            BindObject::UniformBuffer(buffer) | BindObject::StorageBuffer(buffer) => {
                buffer.is_none()
            }
            BindObject::InputAttachment(image_view) => image_view.is_none(),
            BindObject::Texture(image_view, sampler) => image_view.is_none() || sampler.is_none(),
        });

        if has_unbound {
            return Err(MaterialError::IncompleteBindingSet);
        }

        unsafe {
            if self.set_updated {
                self.write_descriptor_set(device);
                self.set_updated = false;
            }

            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                material.pipeline.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                material.pipeline.layout,
                0,
                &[self.descriptor_set],
                &[0],
            )
        };

        Ok(())
    }

    pub fn update_binding(&mut self, binding: usize, object: BindObject) -> MaterialResult<()> {
        use BindObject::*;

        let bindings_len = self.bindings.len();

        let stored = self
            .bindings
            .get_mut(binding)
            .ok_or(MaterialError::BindingOutOfBounds {
                len: bindings_len,
                index: binding,
            })?;

        let object_type = BindingType::from(&object);
        let stored_type = BindingType::from(&*stored);

        match (stored, object) {
            (UniformBuffer(stored), UniformBuffer(object)) => *stored = object,
            (InputAttachment(stored), InputAttachment(object)) => *stored = object,
            (StorageBuffer(stored), StorageBuffer(object)) => *stored = object,
            (Texture(s_image_view, s_sampler), Texture(o_image_view, o_sampler)) => {
                *s_image_view = o_image_view;
                *s_sampler = o_sampler;
            }
            _ => {
                return Err(MaterialError::WrongBindingObjectType {
                    index: binding,
                    provided: object_type,
                    actual: stored_type,
                })
            }
        }

        self.set_updated = true;

        Ok(())
    }

    fn write_descriptor_set(&self, device: &VulkanDevice) {
        let mut writes = Vec::<vk::WriteDescriptorSet>::with_capacity(self.bindings.len());

        let mut buffer_infos = Vec::<vk::DescriptorBufferInfo>::with_capacity(self.bindings.len());
        let mut image_infos = Vec::<vk::DescriptorImageInfo>::with_capacity(self.bindings.len());

        self.bindings
            .iter()
            .enumerate()
            .for_each(|(binding, object)| {
                let mut write = vk::WriteDescriptorSet::default()
                    .dst_set(self.descriptor_set)
                    .dst_binding(binding as u32)
                    .descriptor_type(object.into());

                match object {
                    BindObject::StorageBuffer(Some(buffer))
                    | BindObject::UniformBuffer(Some(buffer)) => {
                        let buffer_info = vk::DescriptorBufferInfo::default()
                            .buffer(*buffer)
                            .offset(0)
                            .range(vk::WHOLE_SIZE);

                        let offset = buffer_infos.len();

                        buffer_infos.push(buffer_info);

                        let ptr = &buffer_infos[offset] as *const _;
                        write.p_buffer_info = ptr;
                    }
                    BindObject::InputAttachment(Some(image_view)) => {
                        let image_info = vk::DescriptorImageInfo::default()
                            .image_view(*image_view)
                            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL);

                        let offset = image_infos.len();
                        image_infos.push(image_info);

                        write.p_image_info = &image_infos[offset] as *const _;
                    }
                    BindObject::Texture(Some(image_view), Some(sampler)) => todo!(),
                    _ => unreachable!(),
                }

                writes.push(write);
            });

        unsafe { device.update_descriptor_sets(&writes, &[]) };
    }
}
