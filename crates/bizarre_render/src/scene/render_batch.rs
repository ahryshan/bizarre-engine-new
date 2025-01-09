use std::{collections::VecDeque, ops::Range};

use crate::mesh::MeshHandle;

use super::render_object::RenderObjectMaterials;

#[derive(Default, Clone)]
pub struct RenderBatch {
    pub mesh: MeshHandle,
    pub materials: RenderObjectMaterials,
    pub offset: usize,
    pub count: usize,
    pub holes: VecDeque<usize>,
}

impl RenderBatch {
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
