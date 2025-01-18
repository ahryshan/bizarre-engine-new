use std::{fmt::Debug, fs::File, io::Read, path::Path};

use bizarre_core::Handle;
use nalgebra_glm::Vec3;
use tobj::LoadOptions;

use crate::{asset_manager::AssetStore, vertex::Vertex};

pub type MeshHandle = Handle<Mesh>;

#[derive(Debug, Default)]
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
        let (models, _) = tobj::load_obj(
            file_path,
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();

        let model = &models[0];

        let positions = model.mesh.positions.chunks(3).map(Vec3::from_column_slice);
        let normals = model.mesh.normals.chunks(3).map(Vec3::from_column_slice);

        let vertices = positions
            .zip(normals)
            .map(|(position, normal)| Vertex {
                position,
                normal,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let indices = model.mesh.indices.clone();

        Self { vertices, indices }
    }
}
