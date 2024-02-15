use glam::vec3;
use glam::Mat4;
use glam::Quat;
use glam::Vec3;

use crate::ecs::component::Component;

#[derive(Debug)]
pub struct PerspectiveProjection {
    pub view: Mat4,
}

impl Component for PerspectiveProjection {}

#[derive(Debug)]
pub struct OrthographicProjection {
    pub view: Mat4,
}

impl Component for OrthographicProjection {}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
}

pub fn clamp(input: f32, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        let view = self.view().to_cols_array();

        vec3(-view[2], -view[6], -view[10])
    }

    pub fn backward(&self) -> Vec3 {
        let view = self.view().to_cols_array();

        vec3(view[2], view[6], view[10])
    }

    pub fn left(&self) -> Vec3 {
        let view = self.view().to_cols_array();

        vec3(-view[0], -view[4], -view[8])
    }

    pub fn right(&self) -> Vec3 {
        let view = self.view().to_cols_array();

        vec3(view[0], view[4], view[8])
    }

    pub fn view(&self) -> Mat4 {
        let view = Mat4::from_rotation_translation(self.rotation.normalize(), self.position);

        view.inverse()
    }

    pub fn yaw(&mut self, amount: f32) {
        self.rotation.y += amount;
    }

    pub fn pitch(&mut self, amount: f32) {
        static LIMIT: f32 = 89_f32 * std::f32::consts::PI / 180_f32;

        self.rotation.x += amount;
        self.rotation.x = clamp(self.rotation.x, -LIMIT, LIMIT);
    }
}

impl Component for Camera {}
