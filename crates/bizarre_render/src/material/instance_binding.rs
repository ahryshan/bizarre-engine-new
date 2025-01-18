use std::collections::BTreeMap;

use ash::vk;

use crate::{buffer::GpuBuffer, shader::ShaderStage};

use super::material_binding::{MaterialBinding, MaterialBindingSet};

pub enum InstanceBinding {
    UniformBuffer(Option<GpuBuffer>),
}

impl From<&MaterialBinding> for InstanceBinding {
    fn from(value: &MaterialBinding) -> Self {
        match value.descriptor_type {
            vk::DescriptorType::UNIFORM_BUFFER => Self::UniformBuffer(None),
            _ => panic!(
                "InstanceBinding: unsupported descriptor type: `${:?}`",
                value.descriptor_type
            ),
        }
    }
}

type SetIndexLocal = usize;

#[derive(Default)]
pub struct MaterialInstanceBindingMap {
    min_set: usize,
    bindings: Vec<InstanceBinding>,
    stage_map: BTreeMap<ShaderStage, Vec<Option<Vec<usize>>>>,
    type_map: BTreeMap<vk::DescriptorType, Vec<(SetIndexLocal, Vec<usize>)>>,
}

impl MaterialInstanceBindingMap {
    pub fn set_binding(
        &mut self,
        stage: ShaderStage,
        set: usize,
        binding: usize,
        object: InstanceBinding,
    ) {
        let index = self.stage_map.get(&stage).unwrap()[set - self.min_set]
            .as_ref()
            .unwrap()[binding];

        self.bindings[index] = object;
    }

    pub fn sets_of_type(
        &self,
        descriptor_type: vk::DescriptorType,
    ) -> Vec<(usize, Vec<&InstanceBinding>)> {
        self.type_map
            .get(&descriptor_type)
            .map(|type_entry| {
                type_entry
                    .iter()
                    .map(|(local_index, bindings)| {
                        let bindings = bindings
                            .iter()
                            .map(|index| &self.bindings[*index])
                            .collect::<Vec<_>>();

                        (local_index + self.min_set, bindings)
                    })
                    .collect()
            })
            .unwrap_or(Vec::new())
    }
}

impl From<&MaterialBindingSet> for MaterialInstanceBindingMap {
    fn from(value: &MaterialBindingSet) -> Self {
        if value.bindings.is_empty() {
            return Self::default();
        }

        let bindings = value
            .bindings
            .iter()
            .map(InstanceBinding::from)
            .collect::<Vec<_>>();

        let (min_set, max_set) = value
            .bindings
            .iter()
            .fold((usize::MAX, usize::MIN), |(min, max), curr| {
                (min.min(curr.set as usize), max.max(curr.set as usize))
            });

        let enumerated_bindings = value.bindings.iter().enumerate().collect::<Vec<_>>();

        let mut type_map = BTreeMap::new();
        let mut stage_map = BTreeMap::new();

        let sets_count = max_set - min_set;

        for (i, binding) in enumerated_bindings {
            binding
                .shader_stage_flags
                .iter_names()
                .for_each(|(_, stage_flag)| {
                    let stage = ShaderStage::from(stage_flag);
                    let stage_sets = stage_map.entry(stage).or_insert(vec![None; sets_count]);
                    let set = stage_sets
                        .get_mut(binding.set as usize)
                        .unwrap()
                        .get_or_insert(Vec::new());

                    set.push(i)
                });

            let type_value = type_map
                .entry(binding.descriptor_type)
                .or_insert(Vec::new());

            let set =
                type_value
                    .iter_mut()
                    .find(|(local_set, _): &&mut (SetIndexLocal, Vec<usize>)| {
                        *local_set == binding.set as usize - min_set
                    });

            match set {
                Some((_, set_bindings)) => set_bindings.push(i),
                None => type_value.push((binding.set as usize - min_set, vec![i])),
            }
        }

        Self {
            min_set,
            bindings,
            stage_map,
            type_map,
        }
    }
}
