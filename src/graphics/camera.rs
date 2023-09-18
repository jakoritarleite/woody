use std::any::TypeId;

use glam::EulerRot;
use glam::Mat4;
use glam::Quat;
use glam::Vec3;

use crate::ecs::component::Bundle;
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

// TODO: refactor camera component, right now its rotating the frustum and not the world, because
// of that the movement is a little weird: when I rotate to left and go forward (+z) it actually
// goes left as if I was doing (-x).
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

impl Bundle for Camera {
    fn components_ids() -> Vec<std::any::TypeId> {
        vec![TypeId::of::<Self>()]
    }

    fn components(
        self,
        storage: &mut crate::ecs::archetypes::ArchetypeStorage,
        row_indexes: &mut impl FnMut(usize),
    ) {
        let row_index = storage.init_component(self);

        row_indexes(row_index);
    }
}
