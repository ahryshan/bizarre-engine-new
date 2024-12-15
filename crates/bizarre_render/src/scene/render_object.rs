use bitflags::bitflags;
use nalgebra_glm::Mat4;

use crate::{material::material_instance::MaterialInstanceHandle, mesh::MeshHandle};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct RenderObjectFlags: u8 {
        const DEFERRED_PASS = 0b0000_0001;
        const FORWARD_PASS = 0b0000_0010;
        const LIGHTING_PASS = 0b000_0100;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderObject {
    pub flags: RenderObjectFlags,
    pub material_instance: MaterialInstanceHandle,
    pub mesh: MeshHandle,
    pub transform: Mat4,
}
