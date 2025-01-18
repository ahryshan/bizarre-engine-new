use ash::vk;
use bizarre_core::Handle;
use material_binding::{MaterialBinding, MaterialBindingSet};
use pipeline::VulkanPipeline;
use thiserror::Error;

pub mod builtin;
pub mod descriptor_buffer;
pub mod instance_binding;
pub mod material_binding;
pub mod material_instance;
pub mod pipeline;
pub mod pipeline_features;

#[derive(Debug, Error)]
pub enum MaterialError {
    #[error(transparent)]
    VkError(#[from] vk::Result),
    #[error("Provided binding index({index}) is out of bounds (len = {len})")]
    BindingOutOfBounds { len: usize, index: usize },
    #[error("Trying to set binding at index {index} to object `{provided:?}` while the the actual binding at this index is `{actual:?}`")]
    WrongBindingObjectType {
        index: usize,
        provided: vk::DescriptorType,
        actual: vk::DescriptorType,
    },
    #[error("Incomplete bindning set")]
    IncompleteBindingSet,
}

pub type MaterialResult<T> = Result<T, MaterialError>;

pub type MaterialHandle = Handle<Material>;

pub struct Material {
    pipeline: VulkanPipeline,
    bindings: MaterialBindingSet,
}

pub struct MaterialCreateInfo {}

impl Material {
    pub fn new(pipeline: VulkanPipeline, bindings: &[MaterialBinding]) -> Self {
        let bindings = MaterialBindingSet::from(bindings.to_vec());

        Self { pipeline, bindings }
    }

    pub(crate) fn pipeline(&self) -> &VulkanPipeline {
        &self.pipeline
    }
}
