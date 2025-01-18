use std::collections::BTreeMap;

use ash::vk;

use crate::{
    shader::{ShaderStage, ShaderStageFlags, ShaderStages},
    vulkan_context::get_device,
};

#[derive(Debug, Clone)]
pub struct MaterialBinding {
    pub set: u32,
    pub binding: u32,
    pub descriptor_type: vk::DescriptorType,
    pub descriptor_count: u32,
    pub binding_rate: MaterialBindingRate,
    pub shader_stage_flags: ShaderStageFlags,
}

impl From<&MaterialBinding> for vk::DescriptorSetLayoutBinding<'_> {
    fn from(value: &MaterialBinding) -> Self {
        vk::DescriptorSetLayoutBinding {
            binding: value.binding,
            descriptor_type: value.descriptor_type,
            descriptor_count: value.descriptor_count,
            stage_flags: value.shader_stage_flags.into(),
            ..Default::default()
        }
    }
}

pub const fn base_scene_bindings() -> &'static [MaterialBinding] {
    &[
        MaterialBinding {
            set: 0,
            binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            binding_rate: MaterialBindingRate::PerFrame,
            shader_stage_flags: ShaderStageFlags::VERTEX,
        },
        MaterialBinding {
            set: 0,
            binding: 1,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            binding_rate: MaterialBindingRate::PerFrame,
            shader_stage_flags: ShaderStageFlags::VERTEX,
        },
    ]
}

#[derive(Debug, Clone, Copy)]
pub enum MaterialBindingRate {
    PerFrame,
    Single,
}

#[derive(Debug, Clone)]
pub struct MaterialBindingSet {
    pub(crate) bindings: Vec<MaterialBinding>,
}

impl From<Vec<MaterialBinding>> for MaterialBindingSet {
    fn from(mut bindings: Vec<MaterialBinding>) -> Self {
        if bindings.is_empty() {
            return Self { bindings };
        }

        bindings.sort_by_cached_key(|binding| {
            (binding.set, binding.binding, binding.shader_stage_flags)
        });

        Self { bindings }
    }
}

pub fn bindings_into_layouts(
    binding_set: &MaterialBindingSet,
) -> Result<Vec<vk::DescriptorSetLayout>, vk::Result> {
    let device = get_device();

    binding_set
        .bindings
        .chunk_by(|a, b| a.set == b.set)
        .map(|bindings| {
            let bindings = bindings
                .iter()
                .map(|binding| vk::DescriptorSetLayoutBinding::from(binding))
                .collect::<Vec<_>>();

            #[cfg(debug_assertions)]
            {
                let first_element_type = bindings[0].descriptor_type;
                debug_assert!(
                    bindings
                        .iter()
                        .all(|binding| binding.descriptor_type == first_element_type),
                    "all bindings in a set must have the same type"
                );
            }

            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings)
                .flags(vk::DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT);

            unsafe { device.create_descriptor_set_layout(&create_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()
}
