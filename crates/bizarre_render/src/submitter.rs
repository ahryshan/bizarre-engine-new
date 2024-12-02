use bizarre_ecs::prelude::*;

use crate::{material::pipeline::PipelineHandle, render_pass::RenderPassHandle};

pub struct RenderPackage {
    pub render_pass: RenderPassHandle,
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
