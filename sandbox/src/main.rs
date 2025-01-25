use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{system::schedule::Schedule, world::ecs_module::EcsModule},
    ecs_modules::sdl_module::SdlModule,
    event::Events,
    prelude::{Res, ResMut, *},
    render::{
        asset_manager::RenderAssets,
        material::builtin::basic_deferred,
        present_target::{PresentError, PresentTargetHandle},
        render_target::RenderTargetHandle,
        renderer::{RenderError, VulkanRenderer},
        scene::{SceneHandle, SceneUniform},
        submitter::RenderPackage,
    },
    sdl::window::{WindowCreateInfo, WindowEvent, WindowPosition, Windows},
};

use nalgebra_glm::{look_at, perspective, Mat4, UVec2, Vec3};
use sandbox_module::SandboxModule;

mod sandbox_module;

struct RenderModule;

#[derive(Resource)]
struct MainPresentTarget(pub PresentTargetHandle);

#[derive(Resource)]
struct MainRenderTarget(pub RenderTargetHandle);

#[derive(Resource)]
struct MainScene(pub SceneHandle);

impl EcsModule for RenderModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        let renderer = VulkanRenderer::new().unwrap();
        let main_window = world
            .resource_mut::<Windows>()
            .unwrap()
            .get_main_window()
            .unwrap();

        let mut assets = RenderAssets::new();

        let present_target_handle =
            assets.create_present_target2(&main_window, renderer.image_count());

        let (width, height) = main_window.size();

        let image_count = renderer.image_count();

        let extent = {
            let (x, y) = main_window.size();
            UVec2::new(x, y)
        };

        let render_target =
            assets.create_swapchain_render_target(extent, image_count, renderer.antialising());

        let mesh = assets.load_mesh("assets/meshes/cube.obj");

        let material = assets.insert_material(basic_deferred());
        let (instance_handle, _) = assets.create_material_instance(material).unwrap();

        let scene_handle = assets.create_scene(image_count);
        let scene = assets.scene_mut(&scene_handle).unwrap();

        let view = look_at(
            &Vec3::new(3.0, 20.0, 5.0),
            &Vec3::zeros(),
            &Vec3::new(0.0, 1.0, 0.0),
        );

        let projection = perspective(
            width as f32 / height as f32,
            90.0f32.to_radians(),
            0.1,
            1000.0,
        );

        scene.update_scene_uniform(SceneUniform { view, projection });

        world.insert_resource(MainPresentTarget(present_target_handle));
        world.insert_resource(MainRenderTarget(render_target));
        world.insert_resource(MainScene(scene_handle));
        world.insert_resource(renderer);
        world.insert_resource(assets);

        world.add_systems(Schedule::Update, render);
    }
}

fn render(
    mut renderer: ResMut<VulkanRenderer>,
    mut assets: ResMut<RenderAssets>,
    mut last_render: Local<Instant>,
    present_target: Res<MainPresentTarget>,
    render_target: Res<MainRenderTarget>,
    scene_handle: Res<MainScene>,
    window_events: Events<WindowEvent>,
    mut skip_render: Local<bool>,
) {
    let elapsed = last_render.elapsed();
    let target = Duration::from_millis(16);

    if elapsed < target {
        std::thread::sleep(target - elapsed);
        *last_render = Instant::now();
    }

    for event in window_events {
        match event {
            WindowEvent::Resized { size, .. } if size.x == 0 || size.y == 0 => *skip_render = true,
            WindowEvent::Resized { handle, size } => {
                let handle = PresentTargetHandle::from_raw(handle.as_raw());
                let present_target = assets.present_target_mut(&handle).unwrap();
                present_target.resize().unwrap();

                assets
                    .render_targets
                    .get_mut(&render_target.0)
                    .unwrap()
                    .resize(size)
                    .unwrap();

                let view = look_at(
                    &Vec3::new(3.0, 2.0, 10.0),
                    &Vec3::zeros(),
                    &Vec3::new(0.0, 1.0, 0.0),
                );

                let projection = perspective(
                    size.x as f32 / size.y as f32,
                    90.0f32.to_radians(),
                    0.1,
                    1000.0,
                );

                assets
                    .scene_mut(&scene_handle.0)
                    .unwrap()
                    .update_scene_uniform(SceneUniform { view, projection });

                *skip_render = false
            }
            WindowEvent::Exposed(..) => *skip_render = false,
            WindowEvent::Hidden(..) => *skip_render = true,
            _ => (),
        }
    }

    if *skip_render {
        return;
    }

    let render_package = RenderPackage {
        pov: Mat4::default(),
        scene: SceneHandle::from_raw(1usize),
    };

    let render_extent = {
        assets
            .present_targets
            .get(&present_target.0)
            .unwrap()
            .size()
    };

    let render_result =
        renderer.render_to_target(&mut assets, render_target.0, render_extent, render_package);

    let present_result = match render_result {
        Ok(()) => renderer.present_to_target(&mut assets, present_target.0, render_target.0),
        Err(RenderError::RenderSkipped) => Err(PresentError::PresentSkipped),
        Err(err) => panic!("Failed render: {err:?}"),
    };

    match present_result {
        Ok(()) | Err(PresentError::PresentSkipped) => {}
        Err(err) => panic!("Failed present: {err:?}"),
    }
}

fn main() -> Result<()> {
    AppBuilder::default()
        .with_name("Bizarre Engine")
        .with_module(
            SdlModule::new().with_main_window(WindowCreateInfo::normal_window(
                "Bizarre Window".into(),
                UVec2::new(800, 600),
                WindowPosition::Undefined,
            )),
        )
        .with_module(RenderModule)
        .with_module(SandboxModule)
        .build()
        .run()
}
