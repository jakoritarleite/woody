use std::sync::Arc;

use ash;
use ash::vk;

use super::Error;

/// Abstraction of the Vulkan CommandBuffer.
pub struct CommandBuffer {
    pub(super) handle: vk::CommandBuffer,
    level: CommandBufferLevel,
    command_pool: vk::CommandPool,
    _device: Arc<ash::Device>,
    // TODO: maybe set command buffer states?
    // NotAllocated, Ready, Recording, InRenderPass, RecordingEnded, Submitted
}

impl CommandBuffer {
    /// Creates a new instance of [`CommandBuffer`].
    fn new(
        level: CommandBufferLevel,
        command_pool: &CommandPool,
        device: Arc<ash::Device>,
    ) -> Result<Self, Error> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool.handle)
            .command_buffer_count(1)
            .level(level.into());

        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info)?[0] };

        log::debug!(
            "Allocated command buffer from pool {:?}",
            command_pool.handle
        );

        Ok(Self {
            handle: command_buffer,
            level,
            command_pool: command_pool.handle,
            _device: device,
        })
    }

    /// Frees the command buffer so it can go back to it's command pool.
    pub fn free(self) {
        unsafe {
            self._device
                .free_command_buffers(self.command_pool, &[self.handle])
        };
    }

    /// Begin recording in this command buffer.
    pub fn begin(&mut self, usage: CommandBufferUsage) -> Result<(), Error> {
        let begin_info = vk::CommandBufferBeginInfo::builder().flags(usage.into());

        unsafe {
            self._device
                .begin_command_buffer(self.handle, &begin_info)?;
        }

        Ok(())
    }

    /// Ends recording in this command buffer.
    pub fn end(&mut self) -> Result<(), Error> {
        unsafe {
            self._device.end_command_buffer(self.handle)?;
        }

        Ok(())
    }

    /// Executes the command buffer on the specified queue.
    pub fn execute(&mut self, queue: vk::Queue) -> Result<(), Error> {
        self.end()?;

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&self.handle));

        unsafe {
            self._device.queue_submit(
                queue,
                std::slice::from_ref(&submit_info),
                vk::Fence::null(),
            )?;

            self._device.queue_wait_idle(queue)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

impl From<CommandBufferLevel> for vk::CommandBufferLevel {
    fn from(val: CommandBufferLevel) -> Self {
        match val {
            CommandBufferLevel::Primary => vk::CommandBufferLevel::PRIMARY,
            CommandBufferLevel::Secondary => vk::CommandBufferLevel::SECONDARY,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandBufferUsage(u32);

bitflags::bitflags! {
    impl CommandBufferUsage: u32 {
        const OneTimeSubmit = vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT.as_raw();
        const RenderPassContinue = vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE.as_raw();
        const SimultaneousUse = vk::CommandBufferUsageFlags::SIMULTANEOUS_USE.as_raw();
        const MultipleSubmit = 0;
    }
}

impl From<CommandBufferUsage> for vk::CommandBufferUsageFlags {
    fn from(value: CommandBufferUsage) -> Self {
        Self::from_raw(value.0)
    }
}

/// Abstraction of the Vulkan CommandPool.
pub struct CommandPool {
    pub(super) handle: vk::CommandPool,
    _device: Arc<ash::Device>,
}

impl CommandPool {
    /// Creates a new instance of [`CommandPool`].
    pub fn new(
        device: Arc<ash::Device>,
        queue_family_index: u32,
        create_flags: CommandPoolCreateFlags,
    ) -> Result<Self, Error> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(create_flags.into());

        let command_pool = unsafe { device.create_command_pool(&create_info, None)? };

        log::debug!(
            "Created command pool for queue index {}",
            queue_family_index
        );

        Ok(Self {
            handle: command_pool,
            _device: device,
        })
    }

    /// Allocates a new [`CommandBuffer`].
    pub fn allocate(
        &self,
        command_buffer_type: CommandBufferLevel,
    ) -> Result<CommandBuffer, Error> {
        CommandBuffer::new(command_buffer_type, self, self._device.clone())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CommandPoolCreateFlags {
    Transient = vk::CommandPoolCreateFlags::TRANSIENT.as_raw(),
    ResetCommandBuffer = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER.as_raw(),
    Protected = vk::CommandPoolCreateFlags::PROTECTED.as_raw(),
}

impl From<CommandPoolCreateFlags> for vk::CommandPoolCreateFlags {
    fn from(value: CommandPoolCreateFlags) -> Self {
        Self::from_raw(value as u32)
    }
}
