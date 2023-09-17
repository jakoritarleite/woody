use std::sync::Arc;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use log::trace;
use vulkano::buffer::Buffer;
use vulkano::buffer::BufferContents;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::Subbuffer;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::Sharing;

use super::GraphicsError;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniformObject {
    pub projection: Mat4,
    pub view: Mat4,
}

impl Default for GlobalUniformObject {
    fn default() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
        }
    }
}

pub struct UniformBuffer<T: ?Sized> {
    handle: Subbuffer<T>,
}

impl<T: BufferContents> UniformBuffer<T> {
    pub fn new(allocator: Arc<StandardMemoryAllocator>) -> Result<Self, GraphicsError> {
        let buffer = Buffer::new_sized(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )?;

        Ok(Self { handle: buffer })
    }

    pub fn handle(&self) -> Subbuffer<T> {
        self.handle.clone()
    }

    pub fn load_data(&mut self, data: T) -> Result<(), GraphicsError> {
        trace!("Loading data into UniformBuffer");

        *(self.handle.write()?) = data;

        Ok(())
    }
}

unsafe impl Zeroable for GlobalUniformObject {}

unsafe impl Zeroable for &GlobalUniformObject {}

unsafe impl Pod for GlobalUniformObject {}

unsafe impl<'a: 'static> Pod for &'a GlobalUniformObject {}
