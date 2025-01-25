use std::{collections::BTreeMap, ops::Deref};

use bitflags::bitflags;

use ash::vk;
use bizarre_core::handle::HandleStrategy;
use bizarre_log::core_trace;

use crate::{
    asset_manager::AssetStore,
    buffer::GpuBuffer,
    mesh::{Mesh, MeshHandle},
    vertex::Vertex,
};

use super::{
    render_batch::RenderBatch, render_object::RenderObject, InstanceData, MeshMapping,
    RenderObjectId, SceneResult, SceneUniform, INITIAL_INDEX_LEN, INITIAL_INDIRECT_LEN,
    INITIAL_INSTANCE_LEN, INITIAL_VERTEX_LEN,
};

bitflags! {
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
    pub struct SceneFrameFlags: u8 {
        const NEED_INSTANCE_DATA_REBUILD        = 0b0000_0001;
        const NEED_INSTANCE_DATA_SYNC           = 0b0000_0010;
        const NEED_MESH_REBUILD                 = 0b0000_0100;
        const NEED_INDIRECT_REBUILD             = 0b0000_1000;
    }
}

#[derive(Clone, Debug)]
pub enum SceneChange {
    AddObject(RenderObjectId, RenderObject),
    UpdateObject(RenderObjectId, InstanceData),
    RemoveObject(RenderObjectId),
    UpdateSceneUniform(SceneUniform),
}

#[derive(Debug)]
pub struct SceneFrameData {
    pub(crate) flags: SceneFrameFlags,
    pub(crate) batches: Vec<RenderBatch>,
    pub(crate) vertex_buffer: GpuBuffer,
    pub(crate) index_buffer: GpuBuffer,
    pub(crate) scene_uniform_buffer: GpuBuffer,
    pub(crate) mesh_map: BTreeMap<MeshHandle, MeshMapping>,
    /// Maps RenderObjectId (throug this vec index) to a (batch_id, index_into_batch) pair
    pub(crate) instance_mapping: Vec<Option<(usize, usize)>>,
    pub(crate) pending_changes: Vec<SceneChange>,
    pub(crate) instance_data: Vec<InstanceData>,
    pub(crate) instance_data_ubo: GpuBuffer,
    pub(crate) indirect_buffer: GpuBuffer,
    pub(crate) indirect_helpers: Vec<u32>,
}

impl SceneFrameData {
    pub fn new() -> SceneResult<Self> {
        let vertex_buffer = GpuBuffer::new(
            (size_of::<Vertex>() * INITIAL_VERTEX_LEN) as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        )?;
        let index_buffer = GpuBuffer::new(
            (size_of::<u32>() * INITIAL_INDEX_LEN) as vk::DeviceSize,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        )?;

        let instance_data_ubo = GpuBuffer::new(
            (size_of::<InstanceData>() * INITIAL_INSTANCE_LEN) as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::HOST_ACCESS_RANDOM,
        )?;

        let scene_uniform_buffer = GpuBuffer::new(
            size_of::<SceneUniform>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::HOST_ACCESS_RANDOM,
        )?;

        let indirect_buffer = GpuBuffer::new(
            (size_of::<vk::DrawIndexedIndirectCommand>() * INITIAL_INDIRECT_LEN) as vk::DeviceSize,
            vk::BufferUsageFlags::INDIRECT_BUFFER,
            vma::MemoryUsage::Auto,
            vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        )?;

        let frame = Self {
            scene_uniform_buffer,
            batches: Vec::default(),
            flags: SceneFrameFlags::empty(),
            vertex_buffer,
            index_buffer,
            indirect_buffer,
            indirect_helpers: Default::default(),
            instance_data: Default::default(),
            instance_data_ubo,
            instance_mapping: Default::default(),
            mesh_map: Default::default(),
            pending_changes: Default::default(),
        };

        Ok(frame)
    }

    pub fn sync_frame_data<S: HandleStrategy<Mesh>>(&mut self, mesh_store: &AssetStore<Mesh, S>) {
        self.pending_changes
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|change| match change {
                SceneChange::AddObject(render_object_id, render_object) => {
                    self.handle_add(render_object_id, render_object)
                }
                SceneChange::UpdateObject(render_object_id, instance_data) => {
                    self.handle_update(render_object_id, instance_data)
                }
                SceneChange::RemoveObject(render_object_id) => self.handle_remove(render_object_id),
                SceneChange::UpdateSceneUniform(uniform) => {
                    self.handle_update_scene_uniform(uniform)
                }
            });

        let flags = self.flags;

        flags.iter().for_each(|flag| match flag {
            SceneFrameFlags::NEED_INDIRECT_REBUILD => self.rebuild_indirects(),
            SceneFrameFlags::NEED_INSTANCE_DATA_SYNC => self.sync_instance_data(),
            SceneFrameFlags::NEED_INSTANCE_DATA_REBUILD => self.rebuild_instance_data(),
            SceneFrameFlags::NEED_MESH_REBUILD => self.rebuild_mesh_data(mesh_store),
            _ => (),
        })
    }

    pub fn add_object(&mut self, object_id: RenderObjectId, object: RenderObject) {
        self.pending_changes
            .push(SceneChange::AddObject(object_id, object));
    }

    pub fn update_object(&mut self, object_id: RenderObjectId, instance_data: InstanceData) {
        self.pending_changes
            .push(SceneChange::UpdateObject(object_id, instance_data))
    }

    pub fn remove_object(&mut self, object_id: RenderObjectId) {
        self.pending_changes
            .push(SceneChange::RemoveObject(object_id))
    }

    pub fn update_scene_uniform(&mut self, uniform: SceneUniform) {
        self.pending_changes
            .push(SceneChange::UpdateSceneUniform(uniform));
    }

    #[inline]
    fn handle_add(&mut self, render_object_id: RenderObjectId, render_object: RenderObject) {
        if render_object_id.0 >= self.instance_mapping.len() {
            let additional: Vec<Option<(usize, usize)>> =
                vec![None; render_object_id.0 - self.instance_mapping.len() + 1];

            self.instance_mapping.extend(additional.into_iter());
        }

        let batch_id = self.batches.iter().position(|batch| {
            batch.mesh == render_object.mesh && batch.materials == render_object.materials
        });

        if let Some(batch_id) = batch_id {
            let batch = &mut self.batches[batch_id];

            if let Some(hole) = batch.holes.pop_front() {
                self.instance_data[batch.offset + hole] = render_object.instance_data;
                self.instance_mapping[render_object_id.0] = Some((batch_id, hole));

                self.flags.insert(SceneFrameFlags::NEED_INSTANCE_DATA_SYNC);
            } else {
                let object_idx = batch.count;
                batch.count += 1;

                self.instance_data
                    .insert(batch.offset + object_idx, render_object.instance_data);

                self.instance_mapping[render_object_id.0] = Some((batch_id, object_idx));

                self.batches[batch_id + 1..]
                    .iter_mut()
                    .for_each(|batch| batch.offset += 1);

                self.flags
                    .insert(SceneFrameFlags::NEED_INSTANCE_DATA_REBUILD);
            }
        } else {
            let batch_id = self.batches.len();

            let offset = match self.batches.last() {
                Some(batch) => batch.offset + batch.count,
                None => 0,
            };

            self.batches.push(RenderBatch {
                offset,
                count: 1,
                holes: Default::default(),
                materials: render_object.materials,
                mesh: render_object.mesh,
            });

            self.flags
                .insert(SceneFrameFlags::NEED_INSTANCE_DATA_REBUILD);

            self.instance_data.push(render_object.instance_data);
            self.instance_mapping[render_object_id.0] = Some((batch_id, 0));

            if !self.mesh_map.contains_key(&render_object.mesh) {
                self.flags.insert(SceneFrameFlags::NEED_MESH_REBUILD);
            }
        }

        self.flags.insert(SceneFrameFlags::NEED_INDIRECT_REBUILD);
    }

    #[inline]
    fn handle_update(&mut self, object_id: RenderObjectId, instance_data: InstanceData) {
        let Some(Some((batch_id, object_id))) = self.instance_mapping.get(object_id.0).cloned()
        else {
            return;
        };

        let Some(batch) = self.batches.get_mut(batch_id) else {
            return;
        };

        let global_offset = batch.offset + object_id;
        self.instance_data[global_offset] = instance_data;

        self.flags.insert(SceneFrameFlags::NEED_INSTANCE_DATA_SYNC);
    }

    #[inline]
    fn handle_remove(&mut self, render_object_id: RenderObjectId) {
        let mapping = self.instance_mapping.get_mut(render_object_id.0);
        let Some(mapping) = mapping else { return };

        let Some((batch_id, object_id)) = mapping else {
            return;
        };

        let batch = self.batches.get_mut(*batch_id);
        let Some(batch) = batch else { return };

        batch.holes.push_back(*object_id);
        *mapping = None;

        self.flags.insert(SceneFrameFlags::NEED_INDIRECT_REBUILD);
    }

    #[inline]
    fn handle_update_scene_uniform(&mut self, uniform: SceneUniform) {
        let mut mapped = self
            .scene_uniform_buffer
            .map_memory::<SceneUniform>(0)
            .unwrap();

        *mapped = uniform;

        drop(mapped);

        self.scene_uniform_buffer
            .flush_range(0, size_of::<SceneUniform>() as vk::DeviceSize)
            .unwrap();
    }

    #[inline]
    fn rebuild_indirects(&mut self) {
        let (helpers, indirects) = self.batches.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut helpers, mut indirects), batch| {
                let Some(mesh_mapping) = self.mesh_map.get(&batch.mesh) else {
                    return (helpers, indirects);
                };

                let first_index = mesh_mapping.index_offset as u32;
                let index_count = mesh_mapping.index_count as u32;
                let vertex_offset = mesh_mapping.vertex_offset as i32;

                let ranges = batch.instance_ranges();
                let command_count = ranges.len();

                for range in ranges.into_iter() {
                    indirects.push(vk::DrawIndexedIndirectCommand {
                        first_index,
                        index_count,
                        vertex_offset,
                        first_instance: (batch.offset + range.start) as u32,
                        instance_count: range.count() as u32,
                    });
                }

                helpers.push(command_count as u32);

                (helpers, indirects)
            },
        );

        {
            {
                let mut mapped_slice = self
                    .indirect_buffer
                    .map_as_slice::<vk::DrawIndexedIndirectCommand>(0, indirects.len())
                    .unwrap();

                mapped_slice.clone_from_slice(&indirects);
            }
            self.indirect_buffer
                .flush_range(0, indirects.len() as vk::DeviceSize)
                .unwrap();
        }

        self.indirect_helpers = helpers;
        self.flags.remove(SceneFrameFlags::NEED_INDIRECT_REBUILD);
    }

    #[inline]
    fn sync_instance_data(&mut self) {
        //TODO: Make it to actually sync only needed ranges
        let mut mapped_slice = self
            .instance_data_ubo
            .map_as_slice(0, self.instance_data.len())
            .unwrap();

        mapped_slice.clone_from_slice(&self.instance_data);

        drop(mapped_slice);

        self.instance_data_ubo
            .flush_range(0, self.instance_data.len() as vk::DeviceSize)
            .unwrap();

        self.flags.remove(SceneFrameFlags::NEED_INSTANCE_DATA_SYNC);
    }

    #[inline]
    fn rebuild_instance_data(&mut self) {
        self.sync_instance_data();

        self.flags
            .remove(SceneFrameFlags::NEED_INSTANCE_DATA_REBUILD);
    }

    #[inline]
    fn rebuild_mesh_data<S: HandleStrategy<Mesh>>(&mut self, mesh_store: &AssetStore<Mesh, S>) {
        let (vertices, indices, mappings) = self.batches.iter().fold(
            (Vec::new(), Vec::new(), BTreeMap::new()),
            |(mut vertices, mut indices, mut mappings), batch| {
                let mesh = mesh_store.get(&batch.mesh).unwrap();
                let mapping = MeshMapping {
                    index_offset: indices.len() as u32,
                    index_count: mesh.indices.len() as u32,
                    vertex_offset: vertices.len() as u32,
                };

                vertices.extend_from_slice(&mesh.vertices);
                indices.extend_from_slice(&mesh.indices);
                mappings.insert(batch.mesh, mapping);

                (vertices, indices, mappings)
            },
        );

        self.mesh_map = mappings;

        {
            let mut mapped_slice = self.vertex_buffer.map_as_slice(0, vertices.len()).unwrap();
            mapped_slice.clone_from_slice(&vertices);
        }

        {
            let mut mapped_slice = self.index_buffer.map_as_slice(0, indices.len()).unwrap();
            mapped_slice.clone_from_slice(&indices);
        }

        self.flags.remove(SceneFrameFlags::NEED_MESH_REBUILD);
    }
}
