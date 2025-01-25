use std::{
    collections::BTreeMap,
    fmt::Debug,
    mem::variant_count,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;
use nalgebra_glm::Mat4;

use crate::{material::material_instance::MaterialInstanceHandle, mesh::MeshHandle};

use super::{object_pass::SceneObjectPass, InstanceData};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct RenderObjectFlags: u8 {
        const DEFERRED_PASS = 0b0000_0001;
        const FORWARD_PASS = 0b0000_0010;
        const LIGHTING_PASS = 0b000_0100;
    }
}

#[derive(Clone)]
pub struct RenderObject<T: Clone> {
    pub meta: RenderObjectMeta,
    pub instance_data: T,
}

impl<T> Debug for RenderObject<T>
where
    T: Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderObject")
            .field("meta", &self.meta)
            .field("instance_data", &self.instance_data)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct RenderObjectMeta {
    pub flags: RenderObjectFlags,
    pub materials: RenderObjectMaterials,
    pub mesh: MeshHandle,
}

impl<T: Clone> RenderObject<T> {
    pub fn new(meta: RenderObjectMeta, instance_data: T) -> Self {
        Self {
            meta,
            instance_data,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderObjectMaterials {
    pub inner: [Option<MaterialInstanceHandle>; variant_count::<SceneObjectPass>()],
}

impl RenderObjectMaterials {
    pub fn new(deferred_material: MaterialInstanceHandle) -> Self {
        Self {
            inner: [Some(deferred_material), None, None],
        }
    }
}

impl Index<SceneObjectPass> for RenderObjectMaterials {
    type Output = Option<MaterialInstanceHandle>;

    fn index(&self, index: SceneObjectPass) -> &Self::Output {
        &self.inner[index as usize]
    }
}

impl IndexMut<SceneObjectPass> for RenderObjectMaterials {
    fn index_mut(&mut self, index: SceneObjectPass) -> &mut Self::Output {
        &mut self.inner[index as usize]
    }
}

impl PartialEq for RenderObjectMaterials {
    fn eq(&self, other: &Self) -> bool {
        self.inner
            .iter()
            .zip(other.inner.iter())
            .all(|(a, b)| a == b)
    }
}

impl Eq for RenderObjectMaterials {}
