use std::time::Instant;

use bizarre_engine::{
    app::app_state::DeltaTime,
    ecs::{commands::Commands, system::schedule::Schedule, world::ecs_module::EcsModule},
    event::Events,
    log::{info, trace},
    prelude::ComponentBatch,
    render::{
        material::{
            builtin::with_basic_deferred, material_instance::MaterialInstanceHandle,
            pipeline::ShaderStageDefinition,
        },
        mesh::MeshHandle,
        render_assets::RenderAssets,
        render_components::camera::{
            Camera, CameraProjection, CameraView, IndependentCameraView, PerspectiveProjection,
            RestrictedCameraView, ViewRestriction,
        },
        scene::{
            render_object::{
                RenderObject, RenderObjectFlags, RenderObjectMaterials, RenderObjectMeta,
            },
            InstanceData, RenderObjectId,
        },
        shader::ShaderStage,
        uniform_block_def,
    },
    sdl::{
        context::with_sdl,
        input::{InputEvent, InputState, MouseButton, Scancode},
        window::Windows,
    },
    util::glm_ext::Vec3Ext,
};

use bizarre_engine::prelude::*;

use nalgebra_glm::{rotate, rotate_x, rotate_y, rotate_z, Mat4, Vec2, Vec3};

use crate::MainScene;

pub struct SandboxModule;

#[derive(Component, Default)]
struct Transform {
    translation: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Transform {
    pub fn transform_matrix(&self) -> Mat4 {
        let mat = Mat4::identity().append_nonuniform_scaling(&self.scale);
        // .append_translation(&self.translation);

        let mat = rotate_x(&mat, self.rotation.x.to_radians());
        let mat = rotate_y(&mat, self.rotation.y.to_radians());
        let mat = rotate_z(&mat, self.rotation.z.to_radians());

        mat.append_translation(&self.translation)
    }
}

#[derive(Component)]
pub struct ColoredCube {
    color: Vec3,
}

#[derive(Component)]
pub struct WhiteCube;

#[derive(Component)]
pub struct PlayerController;

impl EcsModule for SandboxModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        world.add_systems(Schedule::Init, setup_arrows);
        world.add_systems(
            Schedule::Update,
            (show_input_state, control_camera, mouse_grab),
        );
    }
}

pub type SandboxCamera = Camera<RestrictedCameraView<IndependentCameraView>, PerspectiveProjection>;

pub fn default_sandbox_camera(size: Vec2) -> SandboxCamera {
    let view = IndependentCameraView::new(Vec3::new(0.0, 0.0, 10.0), Vec3::zeros());
    let view = RestrictedCameraView::new(
        view,
        ViewRestriction::new(
            Some([-75.0f32.to_radians(), 75.0f32.to_radians()]),
            None,
            Some([0.0, 0.0]),
        ),
    );

    let projection = PerspectiveProjection::new(
        size.x as f32,
        size.y as f32,
        90.0_f32.to_radians(),
        0.001,
        1000.0,
    );

    Camera::new(view, projection)
}

fn mouse_grab(
    mut windows: ResMut<Windows>,
    mut input: ResMut<InputState>,
    mut grabbed: Local<bool>,
    events: Events<InputEvent>,
) {
    let Some(focused_window) = input.mouse_focused_window() else {
        return;
    };
    let Some(focused_window) = windows.window(&focused_window) else {
        return;
    };

    for event in events {
        match event {
            InputEvent::KeyPressed { scancode, .. } if scancode == Scancode::G => {
                input.set_mouse_grab(!*grabbed, focused_window);
                *grabbed = !*grabbed;
            }
            InputEvent::MouseButtonPressed { button, .. }
                if button == MouseButton::Right && !*grabbed =>
            {
                input.set_mouse_grab(true, focused_window);
                *grabbed = true;
            }
            InputEvent::MouseButtonReleased { button, .. }
                if button == MouseButton::Right && *grabbed =>
            {
                input.set_mouse_grab(false, focused_window);
                *grabbed = false;
            }
            _ => {}
        }
    }

    if input.was_key_just_pressed(Scancode::G) {
        let Some(focused_window) = input.mouse_focused_window() else {
            return;
        };
        let Some(focused_window) = windows.window(&focused_window) else {
            return;
        };
    }
}

fn control_camera(
    delta_time: Res<DeltaTime>,
    input: Res<InputState>,
    camera: Query<(&mut SandboxCamera, &PlayerController)>,
) {
    let (camera, _) = camera.into_iter().next().unwrap();

    if input.scroll_delta().y != 0.0 {
        camera.add_zoom(input.scroll_delta().y * 0.01);
    }

    let mouse_grabbed = input.mouse_grabbed();

    if (!mouse_grabbed && input.is_mouse_pressed(MouseButton::Right)) || mouse_grabbed {
        use std::f32::consts::FRAC_PI_2;

        let delta = input.mouse_delta().cast::<f32>();
        let delta = Vec3::new(delta.y.to_radians(), delta.x.to_radians(), 0.0) / 10.0;

        let delta = if mouse_grabbed { -delta } else { delta };

        camera.rotate(&delta);
    }

    const MOVEMENT_SPEED: f32 = 10.0;

    if input.is_key_pressed(Scancode::W) {
        let mut direction = camera.forward();
        direction.y = 0.0;
        direction.normalize_mut();

        let delta = direction * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }
    if input.is_key_pressed(Scancode::S) {
        let mut direction = camera.forward();
        direction.y = 0.0;
        direction.normalize_mut();

        let delta = -direction * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }
    if input.is_key_pressed(Scancode::A) {
        let mut direction = camera.right();
        direction.y = 0.0;
        direction.normalize_mut();

        let delta = -direction * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }
    if input.is_key_pressed(Scancode::D) {
        let mut direction = camera.right();
        direction.y = 0.0;
        direction.normalize_mut();

        let delta = direction * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }
    if input.is_key_pressed(Scancode::Space) {
        let delta = Vec3::UP * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }
    if input.is_key_pressed(Scancode::LShift) {
        let delta = -Vec3::UP * MOVEMENT_SPEED * delta_time.as_secs_f32();
        camera.add_position(&delta);
    }

    const ROLL_SPEED: f32 = 90.0_f32.to_radians();
    if input.is_key_pressed(Scancode::E) {
        let roll_delta = ROLL_SPEED * delta_time.as_secs_f32();
        camera.rotate(&Vec3::new(0.0, 0.0, roll_delta));
    }

    if input.is_key_pressed(Scancode::Q) {
        let roll_delta = -ROLL_SPEED * delta_time.as_secs_f32();
        camera.rotate(&Vec3::new(0.0, 0.0, roll_delta));
    }
}

fn show_input_state(input_state: Res<InputState>) {
    if input_state.was_key_just_pressed(Scancode::I) {
        let pressed_keys = input_state
            .pressed_keys()
            .map(|key| format!("{key}"))
            .collect::<Vec<_>>()
            .join(", ");

        info!("Pressed keys: {pressed_keys}");
    }
}

fn setup_arrows(mut assets: ResMut<RenderAssets>, scene_handle: Res<MainScene>, mut cmd: Commands) {
    let material = with_basic_deferred(|reqs| {
        reqs.stage_definitions[0] = ShaderStageDefinition {
            path: String::from("assets/shaders/cube_deferred.vert"),
            stage: ShaderStage::Vertex,
        };
    });

    let material_handle = assets.insert_material(material);
    let (colored_cube_mat_instance, ..) = assets.create_material_instance(material_handle).unwrap();

    let scene = assets.scene_mut(&scene_handle.0).unwrap();

    let meta = RenderObjectMeta {
        flags: RenderObjectFlags::empty(),
        materials: RenderObjectMaterials::new(colored_cube_mat_instance),
        mesh: MeshHandle::from_raw(0usize),
    };

    let axis_translation = 1.25;
    let main_scale = 1.0;
    let cross_scale = 0.25;

    let mut transform = Transform {
        translation: Vec3::new(axis_translation, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, 0.0),
        scale: Vec3::new(main_scale, cross_scale, cross_scale),
    };

    let x_arrow = RenderObject {
        meta: meta.clone(),
        instance_data: ColoredInstanceData {
            transform: transform.transform_matrix(),
            color: COLORS[0],
        },
    };

    transform.scale = Vec3::new(cross_scale, main_scale, cross_scale);
    transform.translation = Vec3::new(0.0, axis_translation, 0.0);

    let y_arrow = RenderObject {
        meta: meta.clone(),
        instance_data: ColoredInstanceData {
            transform: transform.transform_matrix(),
            color: COLORS[1],
        },
    };

    transform.scale = Vec3::new(cross_scale, cross_scale, main_scale);
    transform.translation = Vec3::new(0.0, 0.0, axis_translation);

    let z_arrow = RenderObject {
        meta: meta.clone(),
        instance_data: ColoredInstanceData {
            transform: transform.transform_matrix(),
            color: COLORS[2],
        },
    };

    scene.add_object(x_arrow);
    scene.add_object(y_arrow);
    scene.add_object(z_arrow);
}

uniform_block_def! {
    struct ColoredInstanceData {
        transform: Mat4,
        color: Vec3,
    }
}

const COLORS: [Vec3; 3] = [
    Vec3::new(0.8, 0.2, 0.2),
    Vec3::new(0.2, 0.8, 0.2),
    Vec3::new(0.2, 0.2, 0.8),
];
