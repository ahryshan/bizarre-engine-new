use cfg_if::cfg_if;
use std::{alloc::Layout, any::TypeId, collections::VecDeque, ops::Range};

use bizarre_core::erased_buffer::ErasedSparseArray;

use crate::mesh::MeshHandle;

use super::render_object::{RenderObjectMaterials, RenderObjectMeta};

#[derive(Debug)]
pub struct RenderBatch {
    pub mesh: MeshHandle,
    pub materials: RenderObjectMaterials,
    pub offset: usize,
    pub count: usize,
    pub instance_data_stride: usize,
    pub instance_data: ErasedSparseArray,
    pub holes: VecDeque<usize>,
}

impl RenderBatch {
    pub fn new(
        offset: usize,
        render_object_meta: &RenderObjectMeta,
        instance_data_layout: Layout,
    ) -> Self {
        let instance_data = unsafe { ErasedSparseArray::from_layout(instance_data_layout) };
        let instance_data_stride = instance_data.stride();

        Self {
            mesh: render_object_meta.mesh,
            materials: render_object_meta.materials.clone(),
            count: 0,
            holes: Default::default(),
            instance_data,
            instance_data_stride,
            offset,
        }
    }

    pub unsafe fn insert<T>(&mut self, at: usize, instance_data: T) {
        if at >= self.instance_data.capacity() {
            self.instance_data.grow(at + 1);
        }

        self.instance_data.insert(at, instance_data);
    }

    pub unsafe fn insert_bytes(&mut self, at: usize, data: &[u8]) {
        if at >= self.instance_data.capacity() {
            self.instance_data.grow(at + 1);
        }

        self.instance_data.insert_bytes(at, data);
    }

    pub fn empty(&self) {
        self.holes.len() == self.count;
    }

    pub fn instance_ranges(&self) -> Vec<Range<usize>> {
        if self.holes.is_empty() {
            let range = 0..self.count;
            if range.is_empty() {
                vec![]
            } else {
                vec![range]
            }
        } else {
            let holes = self.holes.as_slices();
            let mut holes = [holes.0, holes.1].concat();
            holes.sort();

            let mut ranges = Vec::with_capacity(holes.len());

            let mut hole_i = 0;
            let mut instance_i = 0;

            'range_loop: loop {
                if hole_i == holes.len() - 1 {
                    let range = instance_i..holes[hole_i];

                    if !range.is_empty() {
                        ranges.push(range);
                    }

                    let range = holes[hole_i]..self.count;

                    if !range.is_empty() {
                        ranges.push(range);
                    }
                    break 'range_loop;
                }

                if instance_i >= self.count || hole_i >= holes.len() {
                    break 'range_loop;
                }

                let hole = holes[hole_i];
                let range = instance_i..hole;

                if !range.is_empty() {
                    ranges.push(range);
                }

                instance_i = hole + 1;
                hole_i += 1;
            }

            ranges
        }
    }
}
