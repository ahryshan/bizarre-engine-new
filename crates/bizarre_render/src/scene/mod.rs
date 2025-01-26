use std::{
    any::type_name,
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Duration,
};

use ash::vk;
use bitflags::bitflags;
use bizarre_core::{handle::HandleStrategy, Handle};
use bizarre_ecs::prelude::Component;
use bizarre_log::{core_info, core_trace};
use nalgebra_glm::Mat4;
use render_batch::RenderBatch;
use render_object::{RenderObject, RenderObjectMaterials};
use scene_frame::SceneFrameData;
use thiserror::Error;

use bizarre_ecs::prelude::*;

use crate::{
    buffer::{BufferError, GpuBuffer},
    mesh::{Mesh, MeshHandle},
    render_assets::{AssetStore, DenseAssetStore},
    vertex::Vertex,
};

pub mod light;
pub mod object_pass;
pub mod render_batch;
pub mod render_object;
pub mod scene_frame;

pub type SceneHandle = Handle<Scene>;

#[derive(Error, Debug)]
pub enum SceneError {
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error(transparent)]
    VkError(#[from] vk::Result),
}

pub type SceneResult<T> = Result<T, SceneError>;

#[derive(Clone, Copy, Debug, Component)]
pub struct RenderObjectId(usize);

impl RenderObjectId {
    pub fn inner(&self) -> usize {
        self.0
    }
}

const INITIAL_VERTEX_LEN: usize = 10_000;
const INITIAL_INDEX_LEN: usize = 50_000;
const INITIAL_INSTANCE_LEN: usize = 2000;
const INITIAL_INDIRECT_LEN: usize = 1024;

#[repr(C, align(4))]
#[derive(Clone, Debug)]
pub struct SceneUniform {
    pub view: Mat4,
    pub projection: Mat4,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Default)]
pub struct InstanceData {
    pub transform: Mat4,
}

#[derive(Debug)]
pub struct Scene {
    max_frames_in_flight: usize,
    current_frame: usize,

    next_id: usize,
    id_recycling: VecDeque<usize>,

    frames: Vec<SceneFrameData>,
}

macro_rules! trace_sleep {
    ($($input:tt),+$(,)?) => {
        core_trace!($($input),+);
        std::thread::sleep(Duration::from_millis(10))
    };
}

impl Scene {
    pub fn new(max_frames_in_flight: usize) -> SceneResult<Self> {
        let frames = (0..max_frames_in_flight)
            .map(|_| SceneFrameData::new())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            max_frames_in_flight,
            next_id: 0,
            id_recycling: Default::default(),
            current_frame: 0,
            frames,
        })
    }

    pub fn scene_ubo(&self) -> &GpuBuffer {
        &self.frames[self.current_frame].scene_uniform_buffer
    }

    pub fn instance_data_ubo(&self) -> &GpuBuffer {
        &self.frames[self.current_frame].instance_data_ubo
    }

    pub fn vertex_buffer(&self) -> vk::Buffer {
        self.frames[self.current_frame].vertex_buffer.buffer()
    }

    pub fn index_buffer(&self) -> vk::Buffer {
        self.frames[self.current_frame].index_buffer.buffer()
    }

    pub fn sync_frame_data<S, A>(&mut self, mesh_store: &A)
    where
        S: HandleStrategy<Mesh>,
        A: AssetStore<Mesh, S>,
    {
        self.frames[self.current_frame].sync_frame_data(mesh_store)
    }

    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
    }

    pub fn remove_object(&mut self, object_id: RenderObjectId) {
        self.frames
            .iter_mut()
            .for_each(|frame| frame.remove_object(object_id));

        self.id_recycling.push_back(object_id.0)
    }

    #[track_caller]
    pub fn update_object<T: Clone>(&mut self, object_id: RenderObjectId, instance_data: T) {
        assert_ubo_alignemnt::<T>();

        self.frames
            .iter_mut()
            .for_each(|frame| frame.update_object(object_id, instance_data.clone()));
    }

    #[track_caller]
    pub fn add_object<T: Clone>(&mut self, object: RenderObject<T>) -> RenderObjectId {
        assert_ubo_alignemnt::<T>();

        let id = if let Some(id) = self.id_recycling.pop_front() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };

        let id = RenderObjectId(id);

        self.frames
            .iter_mut()
            .for_each(|frame| frame.add_object(id, object.clone()));

        id
    }

    pub fn update_scene_uniform(&mut self, uniform: SceneUniform) {
        self.frames
            .iter_mut()
            .for_each(|frame| frame.update_scene_uniform(uniform.clone()));
    }

    pub fn indirect_draw_iterator(&self) -> (&GpuBuffer, SceneIndirectDrawIterator) {
        let iter = SceneIndirectDrawIterator {
            scene: self,
            batch_offset: 0,
            indirect_offset: 0,
            helper_offset: 0,
            frame_index: self.current_frame,
        };

        (&self.frames[self.current_frame].indirect_buffer, iter)
    }
}

#[derive(Clone, Debug)]
pub struct SceneIndirectDrawIterator<'a> {
    scene: &'a Scene,
    indirect_offset: vk::DeviceSize,
    helper_offset: usize,
    frame_index: usize,
    batch_offset: usize,
}

pub struct IndirectIterItem<'a> {
    pub materials: &'a RenderObjectMaterials,
    pub indirect_offset: vk::DeviceSize,
    pub batch_offset: vk::DeviceSize,
    pub batch_range: vk::DeviceSize,
    pub count: u32,
}

impl<'a> Iterator for SceneIndirectDrawIterator<'a> {
    type Item = IndirectIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let batch = self.scene.frames[self.frame_index]
            .batches
            .get(self.batch_offset)?;

        let helper = self.scene.frames[self.frame_index]
            .indirect_helpers
            .get(self.helper_offset)?;

        let offset = self.indirect_offset;
        self.indirect_offset +=
            (*helper as usize * size_of::<vk::DrawIndexedIndirectCommand>()) as vk::DeviceSize;

        self.batch_offset += 1;
        self.helper_offset += 1;

        Some(IndirectIterItem {
            materials: &batch.materials,
            indirect_offset: offset,
            batch_offset: batch.offset as u64,
            batch_range: (batch.count * batch.instance_data_stride) as u64,
            count: *helper,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MeshMapping {
    index_offset: u32,
    index_count: u32,
    vertex_offset: u32,
}

#[track_caller]
fn assert_ubo_alignemnt<T>() {
    debug_assert!(
        align_of::<T>() == 16,
        "Trying to use `{}` as a instance data for rendering, but it has align of {} (align of 16 is required)",
        type_name::<T>(),
        align_of::<T>()
    );
}
