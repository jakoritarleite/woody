use std::marker::PhantomData;
use std::sync::Arc;

use ash::vk;
use bytemuck::Pod;

use super::command_buffer::{CommandBufferLevel, CommandBufferUsage, CommandPool};
use super::device::Device;
use super::Error;

/// A typed Vulkan buffer with its backing device memory.
pub struct Buffer<T: Pod> {
    pub handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    device: Arc<Device>,
    _marker: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    /// Creates an uninitialized buffer of `count` elements.
    pub fn new(
        device: Arc<Device>,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        count: u64,
        usage: vk::BufferUsageFlags,
        mem_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self, Error> {
        let size = count * std::mem::size_of::<T>() as u64;
        let (handle, memory) = alloc(&device, mem_properties, size, usage, mem_flags)?;
        Ok(Self { handle, memory, size, device, _marker: PhantomData })
    }

    /// Creates a buffer and uploads `data` into it immediately.
    ///
    /// Requires `mem_flags` to include `HOST_VISIBLE` so the memory can be mapped.
    pub fn from_data(
        device: Arc<Device>,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        usage: vk::BufferUsageFlags,
        mem_flags: vk::MemoryPropertyFlags,
        data: &[T],
    ) -> Result<Self, Error> {
        let size = std::mem::size_of_val(data) as vk::DeviceSize;
        let (handle, memory) = alloc(&device, mem_properties, size, usage, mem_flags)?;
        let buf = Self { handle, memory, size, device, _marker: PhantomData };
        buf.write(data)?;
        Ok(buf)
    }

    /// Maps the buffer, writes `data`, then unmaps.
    ///
    /// Buffer must have been created with `HOST_VISIBLE` memory.
    pub fn write(&self, data: &[T]) -> Result<(), Error> {
        let bytes: &[u8] = bytemuck::cast_slice(data);
        unsafe {
            let ptr = self.device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
            self.device.unmap_memory(self.memory);
        }
        Ok(())
    }

    /// Copies `src` into `self` via a one-time-submit command buffer on `queue`.
    pub fn copy_from(
        &self,
        src: &Buffer<T>,
        command_pool: &CommandPool,
        queue: vk::Queue,
    ) -> Result<(), Error> {
        let mut cmd = command_pool.allocate(CommandBufferLevel::Primary)?;
        cmd.begin(CommandBufferUsage::OneTimeSubmit)?;
        let region = vk::BufferCopy { src_offset: 0, dst_offset: 0, size: src.size };
        unsafe {
            self.device.cmd_copy_buffer(cmd.handle, src.handle, self.handle, &[region]);
        }
        cmd.execute(queue)?;
        Ok(())
    }
}

impl<T: Pod> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.handle, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

/// Allocates a `vk::Buffer` and binds it to a suitable `vk::DeviceMemory`.
fn alloc(
    device: &Device,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    mem_flags: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), Error> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

    let reqs = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(reqs.memory_type_bits, mem_flags, mem_properties)?;

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(reqs.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe {
        let mem = device.allocate_memory(&alloc_info, None)?;
        device.bind_buffer_memory(buffer, mem, 0)?;
        mem
    };

    Ok((buffer, memory))
}

fn find_memory_type(
    type_filter: u32,
    required: vk::MemoryPropertyFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> Result<u32, Error> {
    (0..mem_properties.memory_type_count)
        .find(|&i| {
            (type_filter & (1 << i)) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(required)
        })
        .ok_or(Error::NoSuitableMemoryIndex)
}
