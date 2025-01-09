use anyhow::Result;

use bizarre_engine::{
    app::AppBuilder,
    ecs::{system::schedule::Schedule, world::ecs_module::EcsModule},
    ecs_modules::{InputModule, WindowModule},
    event::Events,
    prelude::{Res, ResMut, *},
    render::{
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

use nalgebra_glm::{look_at, perspective_fov, radians, Mat4, UVec2, Vec1, Vec3};

struct RenderModule;

#[derive(Resource)]
struct MainPresentTarget(pub PresentTargetHandle);

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

        let present_target_handle = renderer.create_present_target(&main_window).unwrap();

        let (image_count, width, height) = {
            let present_target = renderer.present_target(&present_target_handle).unwrap();
            let size = present_target.size();

            (present_target.image_count(), size.x, size.y)
        };

        let render_target = renderer
            .create_swapchain_render_target(main_window.size(), image_count)
            .unwrap();

        let scene = renderer.create_scene(image_count as usize).unwrap();

        let mesh = renderer.load_mesh("assets/meshes/cube.obj").unwrap();

        let material = renderer.insert_material(basic_deferred());
        let material_instance = renderer.create_material_instance(material).unwrap();

        let render_object = RenderObject {
            flags: RenderObjectFlags::empty(),
            materials: RenderObjectMaterials::new(material_instance),
            mesh,
            instance_data: InstanceData {
                transform: Mat4::identity(),
            },
        };

        renderer.with_scene_mut(&scene, |scene| {
            let _ = scene.add_object(render_object);

            let view = look_at(
                &Vec3::new(0.0, 2.0, 5.0),
                &Vec3::zeros(),
                &Vec3::new(0.0, 1.0, 0.0),
            );

            let projection = perspective_fov(
                radians(&Vec1::new(90.0)).x,
                width as f32,
                height as f32,
                0.001,
                1000.0,
            );

            scene.update_scene_uniform(SceneUniform { view, projection })
        });

        world.insert_resource(MainPresentTarget(present_target_handle));
        world.insert_resource(MainRenderTarget(render_target));
        world.insert_resource(renderer);

        world.add_systems(Schedule::Update, render);
    }
}

fn render(
    mut renderer: ResMut<VulkanRenderer>,
    present_target: Res<MainPresentTarget>,
    render_target: Res<MainRenderTarget>,
    window_events: Events<WindowEvent>,
) {
    if let Some(window_events) = window_events.as_ref() {
        for event in window_events.iter() {
            if let WindowEvent::Resize { handle, size } = event {
                let present_target = PresentTargetHandle::from_raw(handle.as_raw());
                renderer
                    .resize_present_target(present_target, *size)
                    .unwrap();
            }
        }
    }

    let render_package = RenderPackage {
        pov: Mat4::default(),
        scene: SceneHandle::from_raw(0usize),
    };

    renderer
        .render_to_target(render_target.0, render_package)
        .unwrap();
    renderer
        .present_to_target(present_target.0, render_target.0)
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
