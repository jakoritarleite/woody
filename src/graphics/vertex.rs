use bytemuck::Pod;
use bytemuck::Zeroable;
use nalgebra_glm::Vec2;
use nalgebra_glm::Vec3;
use vulkano::format::Format;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;
use vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription;
use vulkano::pipeline::graphics::vertex_input::VertexInputBindingDescription;
use vulkano::pipeline::graphics::vertex_input::VertexInputRate;

#[repr(C)]
#[derive(Debug, Clone, Copy, VulkanoVertex)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    position: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }

    pub const fn binding_description() -> (u32, VertexInputBindingDescription) {
        (
            0,
            VertexInputBindingDescription {
                stride: std::mem::size_of::<Self>() as u32,
                input_rate: VertexInputRate::Vertex,
            },
        )
    }

    pub const fn attribute_descriptions() -> [(u32, VertexInputAttributeDescription); 1] {
        let position = VertexInputAttributeDescription {
            binding: 0,
            format: Format::R32G32B32_SFLOAT,
            offset: 0,
        };

        [(0, position)]
    }
}

unsafe impl Zeroable for Vertex {}

unsafe impl Zeroable for &Vertex {}

unsafe impl Pod for Vertex {}

unsafe impl<'a: 'static> Pod for &'a Vertex {}
