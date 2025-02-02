use nalgebra_glm::{quat_angle_axis, Quat, Vec3};

pub trait Vec3Ext {
    const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    const DOWN: Vec3 = Vec3::new(0.0, -1.0, 0.0);
    const RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    const LEFT: Vec3 = Vec3::new(-1.0, 0.0, 0.0);
    const FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);
    const BACK: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    const X_AXIS: Vec3 = Self::RIGHT;
    const Y_AXIS: Vec3 = Self::UP;
    const Z_AXIS: Vec3 = Self::BACK;

    fn euler_to_quat(&self) -> Quat;
}

impl Vec3Ext for Vec3 {
    fn euler_to_quat(&self) -> Quat {
        Quat::from_euler(self)
    }
}

pub trait QuatExt {
    fn from_euler(euler: &Vec3) -> Quat {
        quat_angle_axis(euler.z, &Vec3::Z_AXIS)
            * quat_angle_axis(euler.x, &Vec3::X_AXIS)
            * quat_angle_axis(euler.y, &Vec3::Y_AXIS)
    }
}

impl QuatExt for Quat {}
