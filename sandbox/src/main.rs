use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::event::Events;
use bizarre_engine::log::trace;
use bizarre_engine::prelude::*;

use bizarre_engine::render::material::material_binding::MaterialType;
use bizarre_engine::render::material::pipeline::{
    PipelineHandle, ShaderStageDefinition, VulkanPipelineRequirements,
};
use bizarre_engine::render::material::pipeline_features::{
    CullMode, PipelineFeatureFlags, VulkanPipelineFeatures,
};
use bizarre_engine::render::render_pass::{basic_render_pass, RenderPass, RenderPassHandle};
use bizarre_engine::render::render_target::RenderTargetHandle;
use bizarre_engine::render::shader::ShaderKind;
use bizarre_engine::render::submitter::RenderPackage;
use bizarre_engine::window::window_events::WindowEvent;
use bizarre_engine::{
    app::AppBuilder,
    ecs::{
        commands::{Command, Commands},
        system::schedule::Schedule,
        world::ecs_module::EcsModule,
    },
    ecs_modules::{InputModule, WindowModule},
    prelude::{Res, ResMut},
    render::{present_target::PresentTarget, renderer::VulkanRenderer},
    window::{window_manager::WindowManager, WindowCreateInfo},
};

use nalgebra_glm::UVec2;

struct RenderModule;

#[derive(Resource)]
struct MainPresentTarget(pub PresentTarget);

#[derive(Resource)]
struct MainRenderTarget(pub RenderTargetHandle);

impl EcsModule for RenderModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        let mut renderer = VulkanRenderer::new().unwrap();
        let main_window = world
            .resource_mut::<WindowManager>()
            .unwrap()
            .get_main_window()
            .unwrap();

        let (render_target, present_target) = renderer
            .add_window(&main_window, RenderPass::Basic)
            .unwrap();

        world.insert_resource(MainPresentTarget(present_target));
        world.insert_resource(MainRenderTarget(render_target));
        world.insert_resource(renderer);

        world.add_systems(Schedule::Update, render);
    }
}

fn render(
    mut renderer: ResMut<VulkanRenderer>,
    present_target: Res<MainPresentTarget>,
    render_target: Res<MainRenderTarget>,
    mut pipeline: Local<Option<PipelineHandle>>,
    window_events: Events<WindowEvent>,
) {
    if let Some(window_events) = window_events.as_ref() {
        for event in window_events.iter() {
            if let WindowEvent::Resize { handle, size } = event {
                let present_target = PresentTarget::from_raw(handle.as_raw());
                renderer
                    .resize_present_target(present_target, *size)
                    .unwrap();
            }
        }
    }

    let pipeline = pipeline
        .get_or_insert_with(|| {
            let stage_definitions = [
                ShaderStageDefinition {
                    path: "assets/shaders/basic.vert".into(),
                    stage: ShaderKind::Vertex,
                },
                ShaderStageDefinition {
                    path: "assets/shaders/basic.frag".into(),
                    stage: ShaderKind::Fragment,
                },
            ];

            let requirements = VulkanPipelineRequirements {
                features: VulkanPipelineFeatures::default(),
                material_type: MaterialType::Opaque,
                bindings: &[],
                stage_definitions: &stage_definitions,
                render_pass: RenderPass::Basic,
                attachment_count: 1,
                base_pipeline: None,
                vertex_bindings: Box::new([]),
                vertex_attributes: Box::new([]),
            };
            renderer.new_pipeline(&requirements).unwrap()
        })
        .to_owned();

    let render_package = RenderPackage { pipeline };

    renderer.render(render_target.0, render_package).unwrap();
    renderer.present(present_target.0, render_target.0).unwrap();
}

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(InputModule)
        .with_module(
            WindowModule::new().with_main_window(WindowCreateInfo::normal_window(
                "Bizarre Window".into(),
                UVec2::new(800, 600),
            )),
        )
        .with_module(RenderModule)
        .build()
        .run()
}
