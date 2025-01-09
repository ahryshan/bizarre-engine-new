use ash::vk;
use bizarre_core::Handle;
use material_binding::{BindObject, BindObjectSet, BindingSet, BindingType, MaterialBinding};
use pipeline::VulkanPipeline;
use thiserror::Error;

use crate::device::LogicalDevice;

pub mod builtin;
pub mod descriptor_buffer;
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
        provided: BindingType,
        actual: BindingType,
    },
    #[error("Incomplete bindning set")]
    IncompleteBindingSet,
}

pub type MaterialResult<T> = Result<T, MaterialError>;

pub type MaterialHandle = Handle<Material>;

pub struct Material {
    pipeline: VulkanPipeline,
    bindings: BindingSet,
}

pub struct MaterialCreateInfo {}

impl Material {
    pub fn new(pipeline: VulkanPipeline, bindings: &[MaterialBinding]) -> Self {
        let bindings = BindingSet::from(bindings);

        Self { pipeline, bindings }
    }
}
