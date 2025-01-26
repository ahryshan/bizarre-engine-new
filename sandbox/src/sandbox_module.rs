use std::time::Instant;

use bizarre_engine::{
    ecs::{commands::Commands, system::schedule::Schedule, world::ecs_module::EcsModule},
    event::Events,
    log::info,
    prelude::ComponentBatch,
    render::{
        material::{
            builtin::with_basic_deferred, material_instance::MaterialInstanceHandle,
            pipeline::ShaderStageDefinition,
        },
        mesh::MeshHandle,
        render_assets::RenderAssets,
        scene::{
            render_object::{
                RenderObject, RenderObjectFlags, RenderObjectMaterials, RenderObjectMeta,
            },
            InstanceData, RenderObjectId,
        },
        shader::ShaderStage,
        uniform_block_def,
    },
    sdl::input::{InputEvent, InputState, Scancode},
};

use bizarre_engine::prelude::*;

use nalgebra_glm::{rotate, rotate_x, rotate_y, rotate_z, Mat4, Vec3};

use crate::MainScene;

pub struct SandboxModule;

#[derive(Component, Default)]
struct Transform {
    translation: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Transform {
    pub fn get_transform(&self) -> Mat4 {
        let mat = Mat4::identity().append_nonuniform_scaling(&self.scale);
        // .append_translation(&self.translation);

        let mat = rotate_x(&mat, self.rotation.x.to_radians());
        let mat = rotate_y(&mat, self.rotation.y.to_radians());
        let mat = rotate_z(&mat, self.rotation.z.to_radians());

        mat.append_translation(&self.translation)
    }
}

#[derive(ComponentBatch)]
pub struct Cube {
    transform: Transform,
    render_obj: RenderObjectId,
}

impl EcsModule for SandboxModule {
    fn apply(self, world: &mut bizarre_engine::ecs::world::World) {
        world.add_systems(Schedule::Init, setup_cubes);
        world.add_systems(Schedule::Update, (update_cubes, show_input_state));
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

fn setup_cubes(mut assets: ResMut<RenderAssets>, scene_handle: Res<MainScene>, mut cmd: Commands) {
    let material = with_basic_deferred(|reqs| {
        reqs.stage_definitions[0] = ShaderStageDefinition {
            path: String::from("assets/shaders/cube_deferred.vert"),
            stage: ShaderStage::Vertex,
        };
    });

    let material_handle = assets.insert_material(material);
    let (instance_handle, ..) = assets.create_material_instance(material_handle).unwrap();

    let scene = assets.scene_mut(&scene_handle.0).unwrap();

    let quart: i32 = 10;
    let distance: f32 = 3.0;

    for x in -quart..=quart {
        for z in -quart..=quart {
            let transform = Transform {
                translation: Vec3::new(x as f32 * distance, 0.0, z as f32 * distance),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..Default::default()
            };

            let (obj_id, is_colored) = if x.abs() != z.abs() {
                let meta = RenderObjectMeta {
                    flags: RenderObjectFlags::empty(),
                    materials: RenderObjectMaterials::new(instance_handle),
                    mesh: MeshHandle::from_raw(0usize),
                };

                let instance_data = CubeInstanceData {
                    transform: transform.get_transform(),
                    color: COLORS[(x + z) as usize % 3],
                };

                let render_object = RenderObject::new(meta, instance_data);
                (scene.add_object(render_object), true)
            } else {
                let meta = RenderObjectMeta {
                    flags: RenderObjectFlags::empty(),
                    materials: RenderObjectMaterials::new(MaterialInstanceHandle::from_raw(0usize)),
                    mesh: MeshHandle::from_raw(0usize),
                };

                let instance_data = InstanceData {
                    transform: transform.get_transform(),
                };

                let render_object = RenderObject::new(meta, instance_data);
                (scene.add_object(render_object), false)
            };

            cmd.spawn((transform, obj_id, IsColored(is_colored)));
        }
    }
}

#[derive(Component)]
struct IsColored(bool);

fn update_cubes(
    mut last_render: Local<Instant>,
    mut assets: ResMut<RenderAssets>,
    scene_handle: Res<MainScene>,
    cubes: Query<(&mut Transform, &RenderObjectId, &IsColored)>,
) {
    const ROTATION_SPEED_DEG: f32 = 180.0;
    let elapsed = last_render.elapsed();
    *last_render = Instant::now();

    let scene = assets.scene_mut(&scene_handle.0).unwrap();

    for (transform, id, is_colored) in cubes {
        transform.rotation.y += ROTATION_SPEED_DEG * elapsed.as_secs_f32();
        if is_colored.0 {
            scene.update_object(
                *id,
                CubeInstanceData {
                    transform: transform.get_transform(),
                    color: COLORS[(id.inner() * 2) % 3],
                },
            );
        } else {
            scene.update_object(
                *id,
                InstanceData {
                    transform: transform.get_transform(),
                },
            );
        }
    }
}

uniform_block_def! {
    struct CubeInstanceData {
        transform: Mat4,
        color: Vec3,
    }
}

const COLORS: [Vec3; 3] = [
    Vec3::new(0.8, 0.2, 0.2),
    Vec3::new(0.2, 0.8, 0.2),
    Vec3::new(0.2, 0.2, 0.8),
];
