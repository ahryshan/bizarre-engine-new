use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{system::schedule::Schedule, world::ecs_module::EcsModule},
    ecs_modules::sdl_module::SdlModule,
    event::Events,
    prelude::{Res, ResMut, *},
    render::{
        material::builtin::basic_deferred,
        present_target::{self, PresentError, PresentTargetHandle},
        render_assets::{AssetStore, RenderAssets},
        render_components::camera::{Camera, CameraProjection, CameraView, PerspectiveProjection},
        render_target::RenderTargetHandle,
        renderer::{RenderError, VulkanRenderer},
        scene::{SceneHandle, SceneUniform},
        submitter::RenderPackage,
    },
    sdl::window::{WindowCreateInfo, WindowEvent, WindowPosition, Windows},
    util::glm_ext::Vec3Ext,
};

use nalgebra_glm::{look_at, perspective, quat_angle_axis, Mat4, Quat, UVec2, Vec2, Vec3};
use sandbox_module::{default_sandbox_camera, PlayerController, SandboxCamera, SandboxModule};

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
            .main_window()
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

        let size = assets
            .present_targets
            .get(&present_target_handle)
            .unwrap()
            .size();

        let camera = default_sandbox_camera(size.cast::<f32>());

        world.spawn_entity((
            MainRender,
            PlayerController,
            render_target,
            present_target_handle,
            camera,
        ));

        world.insert_resource(MainPresentTarget(present_target_handle));
        world.insert_resource(MainRenderTarget(render_target));
        world.insert_resource(MainScene(scene_handle));
        world.insert_resource(renderer);
        world.insert_resource(assets);

        world.add_systems(Schedule::Preupdate, resize_render_present_targets);
        world.add_systems(Schedule::Update, render);
    }
}

#[derive(Component)]
pub struct MainRender;

fn resize_render_present_targets(
    window_events: Events<WindowEvent>,
    targets: Query<(
        &PresentTargetHandle,
        &RenderTargetHandle,
        &mut SandboxCamera,
    )>,
    mut assets: ResMut<RenderAssets>,
) {
    for event in window_events {
        match event {
            WindowEvent::Resized {
                handle: window_handle,
                size,
            } => {
                if size.x != 0 && size.y != 0 {
                    for (present_handle, render_handle, camera) in targets.iter() {
                        if present_handle != &PresentTargetHandle::from_raw(window_handle.as_raw())
                        {
                            continue;
                        }

                        assets
                            .present_targets
                            .get_mut(present_handle)
                            .unwrap()
                            .resize();

                        assets
                            .render_targets
                            .get_mut(render_handle)
                            .unwrap()
                            .resize(size);

                        camera.resize(&Vec2::new(size.x as f32, size.y as f32));
                    }
                }
            }
            _ => {}
        }
    }
}

fn render(
    mut renderer: ResMut<VulkanRenderer>,
    mut assets: ResMut<RenderAssets>,
    mut last_render: Local<Instant>,
    scene_handle: Res<MainScene>,
    window_events: Events<WindowEvent>,
    mut skip_render: Local<bool>,
    main_render_view: Query<(
        &MainRender,
        &mut SandboxCamera,
        &PresentTargetHandle,
        &RenderTargetHandle,
    )>,
) {
    let elapsed = last_render.elapsed();
    let target = Duration::from_millis(16);

    if elapsed < target {
        std::thread::sleep(target - elapsed);
        *last_render = Instant::now();
    }

    let (main_render, camera, present_target, render_target) =
        main_render_view.iter().next().unwrap();

    let present_target = *present_target;
    let render_target = *render_target;

    for event in window_events {
        match event {
            WindowEvent::Resized { size, .. } if size.x == 0 || size.y == 0 => *skip_render = true,
            WindowEvent::Resized { handle, size } => *skip_render = false,
            WindowEvent::Exposed(..) => *skip_render = false,
            WindowEvent::Hidden(..) => *skip_render = true,
            _ => (),
        }
    }

    if *skip_render {
        return;
    }

    let render_package = RenderPackage {
        scene: SceneHandle::from_raw(0usize),
        view: camera.view_matrix(),
        projection: camera.projection_matrix(),
    };

    let render_extent = { assets.present_targets.get(&present_target).unwrap().size() };

    let render_result =
        renderer.render_to_target(&mut assets, render_target, render_extent, render_package);

    let present_result = match render_result {
        Ok(()) => renderer.present_to_target(&mut assets, present_target, render_target),
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
