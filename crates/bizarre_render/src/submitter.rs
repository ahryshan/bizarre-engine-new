use bizarre_ecs::prelude::*;
use nalgebra_glm::Mat4;

use crate::scene::SceneHandle;

pub struct RenderPackage {
    pub scene: SceneHandle,
    pub view: Mat4,
    pub projection: Mat4,
}

#[derive(Resource)]
pub struct RenderSubmitter {}
