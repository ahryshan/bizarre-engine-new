use std::{fmt::Debug, fs::File, io::Read, path::Path};

use bizarre_core::Handle;
use nalgebra_glm::Vec3;
use tobj::LoadOptions;

use crate::{asset_manager::AssetStore, vertex::Vertex};

pub type MeshHandle = Handle<Mesh>;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn from_vertices(vertices: Vec<Vertex>) -> Self {
        let indices = vertices.iter().enumerate().map(|(i, _)| i as u32).collect();

        Self { vertices, indices }
    }

    pub fn from_vertices_and_indices(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn load_from_obj<P: AsRef<Path> + Debug>(file_path: P) -> Self {
        let (models, _) = tobj::load_obj(file_path, &LoadOptions::default()).unwrap();

        let model = &models[0];

        let vertices = model
            .mesh
            .positions
            .chunks(3)
            .map(|raw_pos| {
                if let [x, y, z] = raw_pos {
                    Vertex {
                        position: Vec3::new(*x, *y, *z),
                    }
                } else {
                    panic!("Trying to compose a vertex but there is not exactly 3 coords");
                }
            })
            .collect();

        let indices = model.mesh.indices.clone();

        Self { vertices, indices }
    }
}
