use std::collections::{BTreeMap, VecDeque};

use ash::vk;
use bizarre_core::handle::HandleStrategy;
use bizarre_log::{core_error, core_warn};
use nalgebra_glm::Mat4;
use object_pass::{SceneObjectPass, SceneObjectPasses};
use render_object::{RenderObject, RenderObjectFlags};
use thiserror::Error;

use crate::{
    asset_manager::AssetStore,
    buffer::{BufferError, GpuBuffer},
    device::VulkanDevice,
    material::material_instance::MaterialInstanceHandle,
    mesh::{Mesh, MeshHandle},
    vertex::Vertex,
};

pub mod light;
pub mod object_pass;
pub mod render_object;

#[derive(Error, Debug)]
pub enum SceneError {
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error(transparent)]
    VkError(#[from] vk::Result),
}

pub type SceneResult<T> = Result<T, SceneError>;

#[derive(Debug)]
pub struct RenderBatch {
    materials: MaterialInstanceHandle,
    mesh: MeshHandle,
    transforms: Vec<Option<Mat4>>,
    holes: VecDeque<usize>,
}

pub struct CompressedBatch {
    pub material_instance: MaterialInstanceHandle,
    pub mesh: MeshHandle,
    pub transforms: Vec<Mat4>,
}

pub struct MeshMapping {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

#[derive(Default)]
pub struct Scene {
    batches: Vec<RenderBatch>,
    object_passes: SceneObjectPasses,
    must_rebuild_geometry_buffers: bool,
    vertex_buffer: Option<GpuBuffer>,
    index_buffer: Option<GpuBuffer>,
    mesh_map: BTreeMap<MeshHandle, MeshMapping>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderObjectId(u32, u32);

#[allow(dead_code)]
impl RenderObjectId {
    pub fn new(batch_id: u32, object_id: u32) -> Self {
        Self(batch_id, object_id)
    }

    pub fn destructure(&self) -> (u32, u32) {
        let Self(batch_id, object_id) = *self;

        (batch_id, object_id)
    }

    pub fn batch_id(&self) -> u32 {
        self.0
    }

    pub fn object_id(&self) -> u32 {
        self.1
    }
}

impl Scene {
    pub fn add_object(&mut self, mut object: RenderObject) -> RenderObjectId {
        let batch = self.batches.iter().position(|batch| {
            batch.materials == object.material_instance && batch.mesh == object.mesh
        });

        let (batch_id, object_id) = if let Some(batch_id) = batch {
            let batch = &mut self.batches[batch_id];
            let object_id = batch.transforms.iter().position(Option::is_none);

            let object_id = if let Some(object_id) = object_id {
                batch.transforms[object_id] = Some(object.transform);
                object_id as u32
            } else {
                let object_id = batch.transforms.len() as u32;
                batch.transforms.push(Some(object.transform));
                object_id
            };

            (batch_id, object_id)
        } else {
            let batch_id = self.batches.len();
            let object_id = 0;

            self.batches.push(RenderBatch {
                materials: object.material_instance,
                mesh: object.mesh,
                transforms: vec![Some(object.transform)],
                holes: Default::default(),
            });

            self.must_rebuild_geometry_buffers = true;

            (batch_id, object_id)
        };

        let id = RenderObjectId::new(batch_id as u32, object_id as u32);

        let deferred = object.flags.intersects(RenderObjectFlags::DEFERRED_PASS);
        let forward = object.flags.intersects(RenderObjectFlags::FORWARD_PASS);

        if deferred && forward {
            core_warn!(
                "Render object has both DEFERRED_PASS and FORWARD_PASS. Adding it only to forward"
            );
            object.flags.remove(RenderObjectFlags::DEFERRED_PASS)
        }

        Vec::<SceneObjectPass>::from(object.flags)
            .iter()
            .for_each(|pass| self.object_passes[*pass].push(id));

        id
    }

    pub fn remove_object(&mut self, id: &RenderObjectId) {
        let (batch_id, object_id) = id.destructure();

        let render_batch = &mut self.batches[batch_id as usize];
        render_batch.transforms[object_id as usize] = None;
        render_batch.holes.push_back(object_id as usize);

        self.object_passes
            .iter_mut()
            .for_each(|pass| pass.retain(|pass_id| pass_id != id));
    }

    pub fn remove_batch(&mut self, batch_id: u32) {
        if self.batches.len() == 0 {
            return;
        }

        let last_batch_id = self.batches.len() - 1;

        if last_batch_id == 0 {
            self.batches.clear();
            self.object_passes.iter_mut().for_each(|pass| pass.clear());
        } else if last_batch_id == batch_id as usize {
            self.batches.remove(batch_id as usize);

            self.object_passes
                .iter_mut()
                .for_each(|pass| pass.retain(|id| id.batch_id() != batch_id));
        } else {
            self.batches.swap_remove(batch_id as usize);

            self.object_passes.iter_mut().for_each(|pass| {
                pass.retain(|id| id.batch_id() != batch_id);

                pass.iter_mut()
                    .filter(|id| id.batch_id() == last_batch_id as u32)
                    .for_each(|id| id.0 = batch_id)
            });
        }

        self.must_rebuild_geometry_buffers = true;
    }

    pub fn compress_batches(&mut self, object_pass: SceneObjectPass) -> Vec<CompressedBatch> {
        compress_batches(&mut self.object_passes[object_pass], &self.batches)
    }

    pub fn must_rebuild_geometry_buffers(&self) -> bool {
        self.must_rebuild_geometry_buffers
    }

    pub fn get_mesh_map<'a, S: HandleStrategy<Mesh>>(
        &'a self,
    ) -> SceneResult<(
        &'a GpuBuffer,
        &'a GpuBuffer,
        &'a BTreeMap<MeshHandle, MeshMapping>,
    )> {
        if self.must_rebuild_geometry_buffers {
            core_error!("Retrieving stale mesh map without rebuilding the buffers!")
        }

        Ok((
            self.vertex_buffer.as_ref().unwrap(),
            self.index_buffer.as_ref().unwrap(),
            &self.mesh_map,
        ))
    }

    pub fn rebuild_buffers<S: HandleStrategy<Mesh>>(
        &mut self,
        device: &VulkanDevice,
        mesh_store: &AssetStore<Mesh, S>,
    ) -> SceneResult<()> {
        let (mesh_map, vertices, indices) = self
            .batches
            .iter()
            .filter_map(|batch| {
                if batch.transforms.len() == batch.holes.len() {
                    return None;
                }

                let mesh = mesh_store.get(&batch.mesh)?;
                let vertices = mesh.vertices.clone();
                let indices = mesh.indices.clone();

                Some((batch.mesh, vertices, indices))
            })
            .fold(
                (BTreeMap::new(), Vec::new(), Vec::new()),
                |(mut map, mut vertices, mut indices),
                 (handle, mut mesh_vertices, mut mesh_indices)| {
                    let mesh_mapping = MeshMapping {
                        vertex_offset: vertices.len() as u32,
                        vertex_count: mesh_vertices.len() as u32,
                        index_offset: indices.len() as u32,
                        index_count: mesh_indices.len() as u32,
                    };

                    map.insert(handle, mesh_mapping);
                    vertices.append(&mut mesh_vertices);
                    indices.append(&mut mesh_indices);
                    (map, vertices, indices)
                },
            );

        let vertex_buffer_size = vertices.len() * size_of::<Vertex>();
        let index_buffer_size = indices.len() * size_of::<u32>();

        let vertex_buffer = self.vertex_buffer.get_or_insert(GpuBuffer::new(
            device,
            vertex_buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::empty(),
        )?);

        let index_buffer = self.index_buffer.get_or_insert(GpuBuffer::new(
            device,
            index_buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::empty(),
        )?);

        if vertex_buffer.size() < vertex_buffer_size {
            vertex_buffer.grow(device, vertex_buffer_size)?;
        }

        if index_buffer.size() < index_buffer_size {
            index_buffer.grow(device, index_buffer_size)?;
        }

        let mut vertex_staging = GpuBuffer::staging_buffer(device, vertex_buffer_size)?;
        let mut index_staging = GpuBuffer::staging_buffer(device, index_buffer_size)?;

        {
            let mut slice = vertex_staging.map_as_slice::<Vertex>(device)?;
            slice.clone_from_slice(&vertices)
        }

        {
            let mut slice = index_staging.map_as_slice::<u32>(device)?;
            slice.copy_from_slice(&indices)
        }

        vertex_buffer.copy_from_buffer(device, &vertex_staging)?;
        index_buffer.copy_from_buffer(device, &vertex_staging)?;

        vertex_staging.destroy(device);
        index_staging.destroy(device);

        self.mesh_map = mesh_map;

        Ok(())
    }
}

#[inline]
fn compress_batches(
    pass_objects: &mut Vec<RenderObjectId>,
    batches: &Vec<RenderBatch>,
) -> Vec<CompressedBatch> {
    pass_objects.sort();
    pass_objects.dedup();

    pass_objects
        .chunk_by(|a, b| a.batch_id() == b.batch_id())
        .map(|object_ids| {
            let batch_id = object_ids[0].batch_id() as usize;

            let RenderBatch {
                materials: material_instance,
                mesh,
                transforms,
                ..
            } = &batches[batch_id];

            let transforms = object_ids
                .iter()
                .filter_map(|object_id| {
                    let object_id = object_id.destructure().1 as usize;

                    if let Some(t) = transforms.get(object_id) {
                        *t
                    } else {
                        None
                    }
                })
                .collect();

            CompressedBatch {
                transforms,
                material_instance: *material_instance,
                mesh: *mesh,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use nalgebra_glm::Mat4;

    use crate::{
        material::MaterialInstanceHandle,
        mesh::MeshHandle,
        scene::{
            render_object::{RenderObject, RenderObjectFlags},
            RenderObjectId, SceneObjectPass,
        },
    };

    use super::Scene;

    #[test]
    fn should_add_to_proper_batches_homogenous_input() {
        let mut scene = Scene::default();

        let render_objects = vec![
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS | RenderObjectFlags::LIGHTING_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            };
            4
        ];

        render_objects.into_iter().for_each(|ro| {
            scene.add_object(ro);
        });

        assert!(scene.batches.len() == 1);
        assert!(scene.deferred_pass_objects.len() == 4);
        assert!(scene.lighting_pass_objects.len() == 4);
        assert!(scene.forward_pass_objects.len() == 0);
    }

    #[test]
    fn should_add_to_proper_batches_heterogenous_input() {
        let mut scene = Scene::default();

        let mut render_objects = vec![
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS | RenderObjectFlags::LIGHTING_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            };
            4
        ];

        render_objects.append(&mut vec![
            RenderObject {
                flags: RenderObjectFlags::FORWARD_PASS | RenderObjectFlags::LIGHTING_PASS,
                material_instance: MaterialInstanceHandle::from_raw(2usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            };
            2
        ]);

        render_objects.append(&mut vec![
            RenderObject {
                flags: RenderObjectFlags::FORWARD_PASS,
                material_instance: MaterialInstanceHandle::from_raw(2usize),
                mesh: MeshHandle::from_raw(2usize),
                transform: Mat4::default(),
            };
            2
        ]);

        render_objects.into_iter().for_each(|ro| {
            scene.add_object(ro);
        });

        assert!(scene.batches.len() == 3);
        assert!(scene.object_passes[SceneObjectPass::Deferred].len() == 4);
        assert!(scene.lighting_pass_objects.len() == 6);
        assert!(scene.forward_pass_objects.len() == 4);
    }

    #[test]
    fn should_compress_batches() {
        let mut scene = Scene::default();

        let render_objects = vec![
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            },
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(2usize),
                transform: Mat4::default(),
            },
            RenderObject {
                flags: RenderObjectFlags::LIGHTING_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(2usize),
                transform: Mat4::default(),
            },
        ];

        render_objects.into_iter().for_each(|ro| {
            scene.add_object(ro);
        });

        let compressed_batches = scene.compress_batches(SceneObjectPass::Deferred);

        assert_eq!(compressed_batches.len(), 2)
    }

    #[test]
    fn should_compress_batches_after_delete() {
        let mut scene = Scene::default();

        let render_objects = vec![
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            },
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            },
            RenderObject {
                flags: RenderObjectFlags::DEFERRED_PASS,
                material_instance: MaterialInstanceHandle::from_raw(1usize),
                mesh: MeshHandle::from_raw(1usize),
                transform: Mat4::default(),
            },
        ];

        render_objects.into_iter().for_each(|ro| {
            scene.add_object(ro);
        });

        scene.remove_object(&RenderObjectId::new(0, 1));

        let compressed_batches = scene.compress_batches(SceneObjectPass::Deferred);

        assert_eq!(compressed_batches[0].transforms.len(), 2)
    }
}
