use nalgebra_glm::Vec3;

#[repr(C)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intesity: f32,
}

#[repr(C)]
pub struct DirectionalLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub fov: f32,
}

pub enum Light {
    Point(PointLight),
    Directional(DirectionalLight),
}
