use crate::ecs::component::Component;

#[derive(Debug)]
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

#[derive(Debug)]
pub struct PerspectiveProjection;
#[derive(Debug)]
pub struct OrthographicProjection;

#[derive(Debug)]
pub struct Camera2d {
    projection: Projection,
}

impl Component for Camera2d {}
