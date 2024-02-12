use std::sync::Arc;

use vulkano::buffer::allocator::SubbufferAllocator;
use vulkano::buffer::allocator::SubbufferAllocatorCreateInfo;
use vulkano::buffer::Buffer as vkBuffer;
use vulkano::buffer::BufferContents;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::BufferCopy;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::CopyBufferInfoTyped;
use vulkano::device::Queue;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::GpuFuture;
use vulkano::sync::Sharing;
use vulkano::DeviceSize;

use crate::graphics::GraphicsError;

use super::command_buffer::CommandBuffer;

pub struct Buffer<T: BufferContents + Clone> {
    handle: Subbuffer<[T]>,
    allocator: Arc<StandardMemoryAllocator>,
    usage: BufferUsage,
    memory_type_filter: MemoryTypeFilter,
    size: u64,
    current_size: u64,
}

impl<T> Buffer<T>
where
    T: BufferContents + Clone,
{
    pub fn new(
        allocator: Arc<StandardMemoryAllocator>,
        usage: BufferUsage,
        memory_type_filter: MemoryTypeFilter,
        size: u64,
    ) -> Result<Self, GraphicsError> {
        let buffer =
            Self::new_buffer_unitialized(allocator.clone(), usage, memory_type_filter, size)?;

        Ok(Self {
            handle: buffer,
            allocator,
            usage,
            memory_type_filter,
            size,
            current_size: 0,
        })
    }

    pub fn new_initialized(
        allocator: Arc<StandardMemoryAllocator>,
        usage: BufferUsage,
        memory_type_filter: MemoryTypeFilter,
        data: &[T],
    ) -> Result<Self, GraphicsError> {
        let buffer =
            Self::new_buffer_initialized(allocator.clone(), usage, memory_type_filter, data)?;

        let size = std::mem::size_of_val(data) as u64;

        Ok(Self {
            handle: buffer,
            allocator,
            usage,
            memory_type_filter,
            size,
            current_size: size,
        })
    }

    pub fn handle(&self) -> &Subbuffer<[T]> {
        &self.handle
    }

    pub fn current_size(&self) -> u64 {
        self.current_size
    }

    pub fn copy_from(
        &mut self,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
        src_buffer: Self,
        src_offset: u64,
        dst_offset: u64,
    ) -> Result<(), GraphicsError> {
        let mut command_buffer =
            CommandBuffer::new(command_buffer_allocator, queue.queue_family_index())?;

        command_buffer.begin(CommandBufferUsage::OneTimeSubmit)?;

        command_buffer
            .handle_mut()?
            .copy_buffer(CopyBufferInfoTyped {
                regions: [BufferCopy {
                    src_offset,
                    dst_offset,
                    size: src_buffer.handle.len(),
                    ..Default::default()
                }]
                .into(),
                ..CopyBufferInfoTyped::buffers(src_buffer.handle, self.handle.clone())
            })?;

        command_buffer
            .end()?
            .execute(queue)?
            .then_signal_fence_and_flush()?
            .wait(None)?;

        Ok(())
    }

    fn new_buffer_initialized(
        allocator: Arc<StandardMemoryAllocator>,
        usage: BufferUsage,
        memory_type_filter: MemoryTypeFilter,
        data: &[T],
    ) -> Result<Subbuffer<[T]>, GraphicsError> {
        let create_info = BufferCreateInfo {
            sharing: Sharing::Exclusive,
            usage,
            ..Default::default()
        };

        let alloc_info = AllocationCreateInfo {
            memory_type_filter,
            ..Default::default()
        };

        let buffer = vkBuffer::from_iter(allocator, create_info, alloc_info, data.iter().cloned())?;

        Ok(buffer)
    }

    fn new_buffer_unitialized(
        allocator: Arc<StandardMemoryAllocator>,
        usage: BufferUsage,
        memory_type_filter: MemoryTypeFilter,
        size: u64,
    ) -> Result<Subbuffer<[T]>, GraphicsError> {
        let create_info = BufferCreateInfo {
            sharing: Sharing::Exclusive,
            usage,
            ..Default::default()
        };

        let alloc_info = AllocationCreateInfo {
            memory_type_filter,
            allocate_preference: vulkano::memory::allocator::MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let buffer =
            vkBuffer::new_unsized::<[T]>(allocator, create_info, alloc_info, size as DeviceSize)?;

        Ok(buffer)
    }
}
