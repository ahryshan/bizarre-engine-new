use std::time::Instant;

use bizarre_engine::{
    ecs::{commands::Commands, system::schedule::Schedule, world::ecs_module::EcsModule},
    event::Events,
    log::info,
    prelude::ComponentBatch,
    render::{
        asset_manager::RenderAssets,
        material::material_instance::MaterialInstanceHandle,
        mesh::MeshHandle,
        scene::{
            render_object::{RenderObject, RenderObjectFlags, RenderObjectMaterials},
            InstanceData, RenderObjectId,
        },
    },
    sdl::input::InputEvent,
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
        world.add_systems(Schedule::Update, (update_cubes));
    }
}

fn setup_cubes(mut assets: ResMut<RenderAssets>, scene_handle: Res<MainScene>, mut cmd: Commands) {
    let scene = assets.scene_mut(&scene_handle.0).unwrap();

    for x in -3..3 {
        for z in -3..3 {
            let transform = Transform {
                translation: Vec3::new(x as f32 * 3.5, 0.0, z as f32 * 3.5),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..Default::default()
            };
            let obj_id = scene.add_object(RenderObject {
                flags: RenderObjectFlags::empty(),
                materials: RenderObjectMaterials::new(MaterialInstanceHandle::from_raw(1usize)),
                mesh: MeshHandle::from_raw(1usize),
                instance_data: InstanceData {
                    transform: transform.get_transform(),
                },
            });

            cmd.spawn(Cube {
                transform,
                render_obj: obj_id,
            });
        }
    }
}

fn update_cubes(
    mut last_render: Local<Instant>,
    mut assets: ResMut<RenderAssets>,
    scene_handle: Res<MainScene>,
    cubes: Query<(&mut Transform, &RenderObjectId)>,
) {
    const ROTATION_SPEED_DEG: f32 = 90.0;
    let elapsed = last_render.elapsed();
    *last_render = Instant::now();

    let scene = assets.scene_mut(&scene_handle.0).unwrap();

    for (transform, id) in cubes {
        transform.rotation.y += ROTATION_SPEED_DEG * elapsed.as_secs_f32();
        scene.update_object(
            *id,
            InstanceData {
                transform: transform.get_transform(),
            },
        );
    }
}
