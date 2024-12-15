use std::{
    mem::variant_count,
    ops::{Index, IndexMut},
};

use crate::material::material_instance::MaterialInstanceHandle;

use super::{render_object::RenderObjectFlags, RenderObjectId};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SceneObjectPass {
    Deferred = 0,
    Forward,
    Lighting,
}

impl From<RenderObjectFlags> for Vec<SceneObjectPass> {
    fn from(value: RenderObjectFlags) -> Self {
        value
            .iter()
            .filter_map(|flag| match flag {
                RenderObjectFlags::DEFERRED_PASS => Some(SceneObjectPass::Deferred),
                RenderObjectFlags::FORWARD_PASS => Some(SceneObjectPass::Forward),
                RenderObjectFlags::LIGHTING_PASS => Some(SceneObjectPass::Lighting),
                _ => None,
            })
            .collect()
    }
}

#[derive(Default, Clone)]
pub struct SceneObjectPasses {
    inner: [Vec<RenderObjectId>; variant_count::<SceneObjectPass>()],
}

impl SceneObjectPasses {
    pub fn iter_mut(&mut self) -> core::slice::IterMut<Vec<RenderObjectId>> {
        self.inner.iter_mut()
    }

    pub fn iter(&mut self) -> core::slice::Iter<Vec<RenderObjectId>> {
        self.inner.iter()
    }
}

impl Index<SceneObjectPass> for SceneObjectPasses {
    type Output = Vec<RenderObjectId>;

    fn index(&self, index: SceneObjectPass) -> &Self::Output {
        &self.inner[index as usize]
    }
}

impl IndexMut<SceneObjectPass> for SceneObjectPasses {
    fn index_mut(&mut self, index: SceneObjectPass) -> &mut Self::Output {
        &mut self.inner[index as usize]
    }
}
