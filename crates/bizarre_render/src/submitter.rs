use bizarre_ecs::prelude::*;

use crate::material::pipeline::PipelineHandle;

pub struct RenderPackage {
    pub pipeline: PipelineHandle,
}

#[derive(Resource)]
pub struct RenderSubmitter {}

impl RenderSubmitter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_package(&mut self) -> RenderPackage {
        todo!()
    }
}
