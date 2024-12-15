use bizarre_core::Handle;

use crate::vertex::Vertex;

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
}
