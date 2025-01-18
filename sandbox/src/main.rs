use std::time::{Duration, Instant};

use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{system::schedule::Schedule, world::ecs_module::EcsModule},
    ecs_modules::{InputModule, WindowModule},
    event::Events,
    log::trace,
    prelude::{Res, ResMut, *},
    render::{
        asset_manager::RenderAssets,
        material::builtin::basic_deferred,
        present_target::PresentTargetHandle,
        render_target::RenderTargetHandle,
        renderer::VulkanRenderer,
        scene::{
            render_object::{RenderObject, RenderObjectFlags, RenderObjectMaterials},
            InstanceData, SceneHandle, SceneUniform,
        },
        submitter::RenderPackage,
    },
    window::{window_events::WindowEvent, window_manager::WindowManager, WindowCreateInfo},
};

use nalgebra_glm::{look_at, perspective, perspective_fov, radians, Mat4, UVec2, Vec1, Vec3};

struct RenderModule;

#[derive(Resource)]
struct MainPresentTarget(pub PresentTargetHandle);

#[derive(Resource)]
struct MainRenderTarget(pub RenderTargetHandle);

impl EcsModule for RenderModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        let renderer = VulkanRenderer::new().unwrap();
        let main_window = world
            .resource_mut::<WindowManager>()
            .unwrap()
            .get_main_window()
            .unwrap();

        let mut assets = RenderAssets::new();

        let present_target_handle =
            assets.create_present_target(&main_window, renderer.image_count());

        let (width, height) = {
            let size = main_window.size();
            (size.x, size.y)
        };

        let image_count = renderer.image_count();

        let render_target = assets.create_swapchain_render_target(
            main_window.size(),
            image_count,
            renderer.antialising(),
        );

        let mesh = assets.load_mesh("assets/meshes/cube.obj");

        let material = assets.insert_material(basic_deferred());
        let (instance_handle, _) = assets.create_material_instance(material).unwrap();

        let render_object = RenderObject {
            flags: RenderObjectFlags::empty(),
            materials: RenderObjectMaterials::new(instance_handle),
            mesh,
            instance_data: InstanceData {
                transform: Mat4::identity(),
            },
        };

        let scene_handle = assets.create_scene(image_count);
        let scene = assets.scene_mut(&scene_handle).unwrap();

        let _ = scene.add_object(render_object);

        let view = look_at(
            &Vec3::new(3.0, 2.0, 10.0),
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
    window_events: Events<WindowEvent>,
) {
    let elapsed = last_render.elapsed();
    let target = Duration::from_millis(16);

    if elapsed < target {
        std::thread::sleep(target - elapsed);
        *last_render = Instant::now();
    }

    if let Some(window_events) = window_events.as_ref() {
        for event in window_events.iter() {
            if let WindowEvent::Resize { handle, size } = event {
                let handle = PresentTargetHandle::from_raw(handle.as_raw());
                let present_target = assets.present_target_mut(&handle).unwrap();
                present_target.resize(*size).unwrap();
            }
        }
    }

    let render_package = RenderPackage {
        pov: Mat4::default(),
        scene: SceneHandle::from_raw(1usize),
    };

    renderer
        .render_to_target(&mut assets, render_target.0, render_package)
        .unwrap();
    renderer
        .present_to_target(&mut assets, present_target.0, render_target.0)
        .unwrap();
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
