use std::{
    f32,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use bizarre_ecs::prelude::*;
use bizarre_log::core_trace;
use bizarre_utils::glm_ext::Vec3Ext;
use nalgebra_glm::{
    look_at, mat4_to_mat3, perspective_fov, perspective_fov_zo, perspective_zo, quat_angle_axis,
    rotate, rotate_vec3, rotate_x_vec3, rotate_y_vec3, rotate_z_vec3, vec2_to_vec3, Mat3, Mat4,
    Quat, TVec3, Vec2, Vec3,
};

pub trait CameraProjection {
    fn projection_matrix(&self) -> Mat4;
    fn zoom(&self) -> f32;
    fn set_zoom(&mut self, zoom: f32);
    fn add_zoom(&mut self, delta: f32);
    fn resize(&mut self, size: &Vec2);
}

impl<D, P> CameraProjection for D
where
    D: Deref<Target = P> + DerefMut,
    P: CameraProjection,
{
    fn projection_matrix(&self) -> Mat4 {
        self.deref().projection_matrix()
    }

    fn zoom(&self) -> f32 {
        self.deref().zoom()
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.deref_mut().set_zoom(zoom);
    }

    fn add_zoom(&mut self, delta: f32) {
        self.deref_mut().add_zoom(delta);
    }

    fn resize(&mut self, size: &Vec2) {
        self.deref_mut().resize(size);
    }
}

pub trait CameraView {
    fn view_matrix(&self) -> Mat4;
    fn forward(&self) -> Vec3;
    fn right(&self) -> Vec3;
    fn up(&self) -> Vec3;

    fn rotation(&self) -> Vec3;
    fn orientation(&self) -> Quat;

    /// Linear motion in camera's view plane
    fn pan(&mut self, delta: &Vec2);

    /// Rotation around camera's axes
    fn rotate(&mut self, delta: &Vec3);

    fn set_rotation(&mut self, rotation: &Vec3);

    /// Camera's eye position
    fn eye(&self) -> Vec3;

    /// Set camera eye's position
    fn set_position(&mut self, position: &Vec3);

    /// Add to the camera eye's position
    fn add_position(&mut self, delta: &Vec3);
}

impl<D, V> CameraView for D
where
    D: Deref<Target = V> + DerefMut,
    V: CameraView,
{
    fn view_matrix(&self) -> Mat4 {
        self.deref().view_matrix()
    }

    fn forward(&self) -> Vec3 {
        self.deref().forward()
    }

    fn right(&self) -> Vec3 {
        self.deref().right()
    }

    fn up(&self) -> Vec3 {
        self.deref().up()
    }

    fn rotation(&self) -> Vec3 {
        self.deref().rotation()
    }

    fn orientation(&self) -> Quat {
        self.deref().orientation()
    }

    fn pan(&mut self, delta: &Vec2) {
        self.deref_mut().pan(delta);
    }

    fn rotate(&mut self, delta: &Vec3) {
        self.deref_mut().rotate(delta)
    }

    fn set_rotation(&mut self, rotation: &Vec3) {
        self.deref_mut().set_rotation(rotation);
    }

    fn eye(&self) -> Vec3 {
        self.deref().eye()
    }

    fn set_position(&mut self, position: &Vec3) {
        self.deref_mut().set_position(position);
    }

    fn add_position(&mut self, delta: &Vec3) {
        self.deref_mut().add_position(delta);
    }
}

pub struct Camera<V, P> {
    view: V,
    projection: P,
}

impl<V: 'static, P: 'static> Resource for Camera<V, P> {}
impl<V: 'static, P: 'static> Component for Camera<V, P> {}

impl<V: CameraView, P: CameraProjection> Camera<V, P> {
    pub fn new(view: V, projection: P) -> Self {
        Self { view, projection }
    }

    pub fn view(&self) -> &V {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut V {
        &mut self.view
    }

    pub fn projection(&self) -> &P {
        &self.projection
    }

    pub fn projection_mut(&mut self) -> &mut P {
        &mut self.projection
    }
}

impl<V: CameraView, P> CameraView for Camera<V, P> {
    fn view_matrix(&self) -> Mat4 {
        self.view.view_matrix()
    }

    fn forward(&self) -> Vec3 {
        self.view.forward()
    }

    fn right(&self) -> Vec3 {
        self.view.right()
    }

    fn up(&self) -> Vec3 {
        self.view.up()
    }

    fn rotation(&self) -> Vec3 {
        self.view.rotation()
    }

    fn orientation(&self) -> Quat {
        self.view.orientation()
    }

    fn pan(&mut self, delta: &Vec2) {
        self.view.pan(delta)
    }

    fn rotate(&mut self, delta: &Vec3) {
        self.view.rotate(delta)
    }

    fn set_rotation(&mut self, rotation: &Vec3) {
        self.view.set_rotation(rotation);
    }

    fn eye(&self) -> Vec3 {
        self.eye()
    }

    fn set_position(&mut self, position: &Vec3) {
        self.view.set_position(position);
    }

    fn add_position(&mut self, delta: &Vec3) {
        self.view.add_position(delta);
    }
}

impl<V, P: CameraProjection> CameraProjection for Camera<V, P> {
    fn projection_matrix(&self) -> Mat4 {
        self.projection.projection_matrix()
    }

    fn zoom(&self) -> f32 {
        self.projection.zoom()
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.projection.set_zoom(zoom);
    }

    fn add_zoom(&mut self, delta: f32) {
        self.projection.add_zoom(delta)
    }

    fn resize(&mut self, size: &Vec2) {
        self.projection.resize(size);
    }
}

impl<V, P> Clone for Camera<V, P>
where
    V: CameraView + Clone,
    P: CameraProjection + Clone,
{
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            projection: self.projection.clone(),
        }
    }
}

impl<V, P> Debug for Camera<V, P>
where
    V: Debug,
    P: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Camera")
            .field("view", &self.view)
            .field("projection", &self.projection)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct OrthogonalProjection {}

impl CameraProjection for OrthogonalProjection {
    fn projection_matrix(&self) -> Mat4 {
        todo!()
    }

    fn zoom(&self) -> f32 {
        todo!()
    }

    fn set_zoom(&mut self, zoom: f32) {
        todo!()
    }

    fn add_zoom(&mut self, delta: f32) {
        todo!()
    }

    fn resize(&mut self, size: &Vec2) {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct PerspectiveProjection {
    fovy: f32,
    width: f32,
    height: f32,
    near: f32,
    far: f32,
    zoom: f32,
}

impl PerspectiveProjection {
    pub fn new(fovy: f32, width: f32, height: f32, near: f32, far: f32) -> Self {
        Self {
            fovy,
            width,
            height,
            near,
            far,
            zoom: 1.0,
        }
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.zoom = self.min_zoom().max(self.zoom);
    }

    fn min_zoom(&mut self) -> f32 {
        std::f32::consts::PI / self.fovy
    }
}

impl CameraProjection for PerspectiveProjection {
    fn projection_matrix(&self) -> Mat4 {
        let Self {
            fovy,
            width,
            height,
            near,
            far,
            zoom,
        } = self;

        perspective_zo(width / height, fovy / zoom, *near, *far)
    }

    fn zoom(&self) -> f32 {
        self.zoom
    }

    fn set_zoom(&mut self, zoom: f32) {
        let min_zoom = self.min_zoom();
        self.zoom = min_zoom.max(zoom);
    }

    fn add_zoom(&mut self, delta: f32) {
        self.set_zoom(self.zoom + delta);
    }

    fn resize(&mut self, size: &Vec2) {
        self.width = size.x;
        self.height = size.y;
    }
}

#[derive(Clone, Debug, Component)]
pub struct IndependentCameraView {
    position: Vec3,
    rotation: Vec3,
}

impl IndependentCameraView {
    pub fn new(position: Vec3, rotation: Vec3) -> Self {
        Self { position, rotation }
    }

    pub fn rotation_matrix(&self) -> Mat3 {
        let m = rotate(&Mat4::identity(), self.rotation.x, &Vec3::X_AXIS);
        let m = rotate(&m, self.rotation.y, &Vec3::Y_AXIS);
        let m = rotate(&m, self.rotation.z, &Vec3::Z_AXIS);
        let m = m.normalize();
        mat4_to_mat3(&m)
    }
}

impl CameraView for IndependentCameraView {
    fn view_matrix(&self) -> Mat4 {
        let rotation = self.rotation_matrix();
        let forward = rotation * Vec3::FORWARD;
        let up = rotation * Vec3::UP;

        look_at(&self.position, &(self.position + forward), &up)
    }

    fn forward(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::FORWARD
    }

    fn right(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::RIGHT
    }

    fn up(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::UP
    }

    fn rotation(&self) -> Vec3 {
        self.rotation
    }

    fn orientation(&self) -> Quat {
        quat_angle_axis(self.rotation.x, &Vec3::X_AXIS)
            * quat_angle_axis(self.rotation.y, &Vec3::Y_AXIS)
            * quat_angle_axis(self.rotation.z, &Vec3::Z_AXIS)
    }

    fn pan(&mut self, delta: &Vec2) {
        self.position += self.rotation_matrix() * Vec3::new(delta.x, delta.y, 0.0);
    }

    fn rotate(&mut self, delta: &Vec3) {
        self.rotation += delta;
    }

    fn set_rotation(&mut self, rotation: &Vec3) {
        self.rotation = rotation.clone();
    }

    fn eye(&self) -> Vec3 {
        self.position
    }

    fn set_position(&mut self, position: &Vec3) {
        self.position = position.clone();
    }

    fn add_position(&mut self, delta: &Vec3) {
        self.position += delta;
    }
}

pub struct TargetedCameraView {
    target: Vec3,
    distance: f32,
    rotation: Vec3,
}

impl TargetedCameraView {
    pub fn new(target: Vec3, distance: f32, rotation: Vec3) -> Self {
        Self {
            target,
            distance,
            rotation,
        }
    }

    pub fn rotation_matrix(&self) -> Mat3 {
        let m = rotate(&Mat4::identity(), self.rotation.x, &Vec3::X_AXIS);
        let m = rotate(&m, self.rotation.y, &Vec3::Y_AXIS);
        let m = rotate(&m, self.rotation.z, &Vec3::Z_AXIS);
        let m = m.normalize();
        mat4_to_mat3(&m)
    }
}

impl CameraView for TargetedCameraView {
    fn view_matrix(&self) -> Mat4 {
        let rotation = self.rotation_matrix();
        let arm = rotation * Vec3::BACK.scale(self.distance);
        let eye = self.target + arm;
        let up = rotation * Vec3::UP;

        look_at(&eye, &self.target, &up)
    }

    fn forward(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::FORWARD
    }

    fn right(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::RIGHT
    }

    fn up(&self) -> Vec3 {
        self.rotation_matrix() * Vec3::RIGHT
    }

    fn rotation(&self) -> Vec3 {
        self.rotation
    }

    fn orientation(&self) -> Quat {
        quat_angle_axis(self.rotation.x, &Vec3::X_AXIS)
            * quat_angle_axis(self.rotation.y, &Vec3::Y_AXIS)
            * quat_angle_axis(self.rotation.z, &Vec3::Z_AXIS)
    }

    fn pan(&mut self, delta: &Vec2) {
        self.target += self.rotation_matrix() * vec2_to_vec3(delta);
    }

    fn rotate(&mut self, delta: &Vec3) {
        self.rotation += delta;
    }

    fn set_rotation(&mut self, rotation: &Vec3) {
        self.rotation = rotation.clone()
    }

    fn eye(&self) -> Vec3 {
        let arm = self.rotation_matrix() * Vec3::BACK.scale(self.distance);
        self.target + arm
    }

    fn set_position(&mut self, position: &Vec3) {
        let position_delta = self.eye() - position;
        self.target += position
    }

    fn add_position(&mut self, delta: &Vec3) {
        self.target += delta;
    }
}

pub type ViewRestriction = TVec3<Option<[f32; 2]>>;

pub struct RestrictedCameraView<V> {
    view: V,
    restriction: ViewRestriction,
}

impl<V: CameraView> RestrictedCameraView<V> {
    pub fn new(view: V, restriction: ViewRestriction) -> Self {
        Self { view, restriction }
    }

    pub fn update_restrictions<F>(&mut self, f: F)
    where
        F: Fn(&mut ViewRestriction),
    {
        f(&mut self.restriction);
        self.enforce_restrictions();
    }

    pub fn enforce_restrictions(&mut self) {
        let mut rotation = self.view.rotation();

        const PI_2: f32 = f32::consts::PI * 2.0;

        if let Some(r) = self.restriction.x {
            if r[0] == r[1] {
                rotation.x = r[0];
            } else {
                rotation.x = (rotation.x % PI_2).clamp(r[0], r[1]);
            }
        }
        if let Some(r) = self.restriction.y {
            if r[0] == r[1] {
                rotation.y = r[0];
            } else {
                rotation.y = (rotation.y % PI_2).clamp(r[0], r[1]);
            }
        }
        if let Some(r) = self.restriction.z {
            if r[0] == r[1] {
                rotation.z = r[0]
            } else {
                rotation.z = (rotation.z % PI_2).clamp(r[0], r[1]);
            }
        }

        self.view.set_rotation(&rotation);
    }
}

impl<V: CameraView> CameraView for RestrictedCameraView<V> {
    fn view_matrix(&self) -> Mat4 {
        self.view.view_matrix()
    }

    fn forward(&self) -> Vec3 {
        self.view.forward()
    }

    fn right(&self) -> Vec3 {
        self.view.right()
    }

    fn up(&self) -> Vec3 {
        self.view.up()
    }

    fn rotation(&self) -> Vec3 {
        self.view.rotation()
    }

    fn orientation(&self) -> Quat {
        self.view.orientation()
    }

    fn pan(&mut self, delta: &Vec2) {
        self.view.pan(delta);
    }

    fn rotate(&mut self, delta: &Vec3) {
        self.view.rotate(delta);
        self.enforce_restrictions();
    }

    fn set_rotation(&mut self, rotation: &Vec3) {
        self.view.set_rotation(rotation);
        self.enforce_restrictions();
    }

    fn eye(&self) -> Vec3 {
        self.view.eye()
    }

    fn set_position(&mut self, position: &Vec3) {
        self.view.set_position(position);
    }

    fn add_position(&mut self, delta: &Vec3) {
        self.view.add_position(delta);
    }
}
