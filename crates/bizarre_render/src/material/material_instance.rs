use ash::vk;
use bizarre_core::Handle;

use super::{
    instance_binding::MaterialInstanceBindingMap, Material, MaterialHandle, MaterialResult,
};

pub type MaterialInstanceHandle = Handle<MaterialInstance>;

pub struct MaterialInstance {
    pub(crate) material_handle: MaterialHandle,
    pub(crate) bind_map: MaterialInstanceBindingMap,
}

impl MaterialInstance {
    pub(crate) fn new(
        material_handle: MaterialHandle,
        material: &Material,
    ) -> MaterialResult<Self> {
        let bind_map = MaterialInstanceBindingMap::from(&material.bindings);

        Ok(Self {
            material_handle,
            bind_map,
        })
    }

    pub fn material_handle(&self) -> MaterialHandle {
        self.material_handle
    }
}
