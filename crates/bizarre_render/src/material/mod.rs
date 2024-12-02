use bizarre_core::Handle;
use material_binding::{BindObjectSet, BindingSet};
use pipeline::VulkanPipeline;

pub mod material_binding;
pub mod pipeline;
pub mod pipeline_features;

pub type MaterialHandle = Handle<Material>;

pub struct Material {
    pipeline: VulkanPipeline,
    bindings: BindingSet,
}

impl Material {
    pub fn new(pipeline: VulkanPipeline, bindings: BindingSet) -> Self {
        Self { pipeline, bindings }
    }
}

pub type MatrialInstance = Handle<MaterialInstance>;

pub struct MaterialInstance {
    material: MaterialHandle,
    bindings: BindObjectSet,
}
