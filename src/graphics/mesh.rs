use std::sync::Arc;

use nalgebra_glm::vec2;
use nalgebra_glm::vec3;
use nalgebra_glm::Vec2;
use nalgebra_glm::Vec3;
use vulkano::buffer::Buffer;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::IndexBuffer;
use vulkano::buffer::Subbuffer;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::Swapchain;
use vulkano::sync::Sharing;
use vulkano::DeviceSize;

use crate::ecs::component::Component;

use super::context::Graphics;
use super::vertex::Vertex;
use super::GraphicsError;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub vbuffer: Subbuffer<[Vertex]>,
    pub ibuffer: IndexBuffer,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl Mesh {
    pub(crate) fn create_vbuffer(
        vertices: Vec<Vertex>,
        memory_allocator: &StandardMemoryAllocator,
    ) -> Result<Subbuffer<[Vertex]>, GraphicsError> {
        let subbuffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                sharing: Sharing::Exclusive,
                usage: BufferUsage::TRANSFER_SRC | BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.into_iter(),
        )?;

        Ok(subbuffer)
    }

    pub(crate) fn create_ibuffer(
        indexes: Vec<u16>,
        memory_allocator: &StandardMemoryAllocator,
    ) -> Result<IndexBuffer, GraphicsError> {
        let index_subbuffer = Buffer::new_slice(
            memory_allocator,
            BufferCreateInfo {
                sharing: Sharing::Exclusive,
                usage: BufferUsage::TRANSFER_SRC | BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indexes.len() as DeviceSize,
        )?;
        index_subbuffer.write()?.copy_from_slice(&indexes);

        let ibuffer = IndexBuffer::from(index_subbuffer);

        Ok(ibuffer)
    }
}

impl Graphics {
    pub(crate) fn push_mesh(&mut self, mesh: impl IntoMesh) -> Result<(), GraphicsError> {
        let mesh = mesh.into_mesh(
            &self.vulkan.memory_allocator,
            self.window.inner_size().into(),
        )?;

        self.meshes.push(mesh);

        Ok(())
    }
}

pub trait IntoMesh {
    fn into_mesh(
        &self,
        memory_allocator: &StandardMemoryAllocator,
        extent: [f32; 2],
    ) -> Result<Mesh, GraphicsError>;
}

impl IntoMesh for Mesh {
    fn into_mesh(
        &self,
        _memory_allocator: &StandardMemoryAllocator,
        _swapchain: [f32; 2],
    ) -> Result<Mesh, GraphicsError> {
        Ok(self.clone())
    }
}

impl IntoMesh for [Vertex; 3] {
    fn into_mesh(
        &self,
        memory_allocator: &StandardMemoryAllocator,
        _swapchain: [f32; 2],
    ) -> Result<Mesh, GraphicsError> {
        let vbuffer = Mesh::create_vbuffer(self.to_vec(), memory_allocator)?;
        let ibuffer = Mesh::create_ibuffer(vec![0, 1, 2], memory_allocator)?;

        Ok(Mesh {
            vertices: self.to_vec(),
            vbuffer,
            ibuffer,
            vertex_count: self.len(),
            index_count: 3,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    size: Vec2,
    color: Vec3,
}

impl Rectangle {
    pub fn new(size: Vec2, color: Vec3) -> Self {
        Self { size, color }
    }
}

impl IntoMesh for Rectangle {
    fn into_mesh(
        &self,
        memory_allocator: &StandardMemoryAllocator,
        extent: [f32; 2],
    ) -> Result<Mesh, GraphicsError> {
        let width = extent[0];
        let height = extent[1];

        // TODO use a uniform buffer to do the transform into NDC
        let ndc_x = |x: f32| (2.0 * x) / width - 1.0;
        let ndc_y = |y: f32| (2.0 * y) / height - 1.0;

        let a = vec2(ndc_x(0.0), ndc_y(0.0));
        let b = vec2(ndc_x(0.0), ndc_y(self.size.y));
        let c = vec2(ndc_x(self.size.x), ndc_y(self.size.y));
        let d = vec2(ndc_x(self.size.x), ndc_y(0.0));

        let vertices = vec![
            Vertex::new(a, self.color),
            Vertex::new(b, self.color),
            Vertex::new(c, self.color),
            Vertex::new(d, self.color),
            //Vertex::new(b, vec3(1.0, 1.0, 0.0)),
            //Vertex::new(c, vec3(0.0, 0.0, 1.0)),
            //Vertex::new(d, vec3(0.0, 1.0, 1.0)),
        ];

        let vbuffer = Mesh::create_vbuffer(vertices.clone(), memory_allocator)?;
        let ibuffer = Mesh::create_ibuffer(vec![0, 1, 2, 0, 2, 3], memory_allocator)?;

        Ok(Mesh {
            vertex_count: vertices.len(),
            index_count: 6,
            vertices,
            vbuffer,
            ibuffer,
        })
    }
}

impl Component for Rectangle {}
