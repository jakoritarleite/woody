use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

use crate::graphics::GraphicsError;

pub enum CommandBufferState {
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
}

type Allocator = Arc<StandardCommandBufferAllocator>;

pub struct CommandBuffer {
    handle: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Allocator>, Allocator>>,
    allocator: Allocator,
    queue_family_index: u32,
    pub(super) state: CommandBufferState,
}

impl CommandBuffer {
    pub fn new(
        allocator: Arc<StandardCommandBufferAllocator>,
        queue_family_index: u32,
    ) -> Result<Self, GraphicsError> {
        Ok(Self {
            handle: None,
            allocator,
            queue_family_index,
            state: CommandBufferState::Ready,
        })
    }

    pub fn begin(&mut self, usage: CommandBufferUsage) -> Result<(), GraphicsError> {
        let builder =
            AutoCommandBufferBuilder::primary(&self.allocator, self.queue_family_index, usage)?;

        self.handle = Some(builder);
        self.state = CommandBufferState::Recording;

        Ok(())
    }

    pub fn end(&mut self) -> Result<Arc<PrimaryAutoCommandBuffer<Allocator>>, GraphicsError> {
        if let Some(handle) = self.handle.take() {
            let command_buffer = handle.build()?;
            self.state = CommandBufferState::RecordingEnded;

            return Ok(command_buffer);
        }

        Err(GraphicsError::InvalidCommandBufferUsage(
            "end command buffer that is not allocated",
        ))
    }

    pub fn handle_mut(
        &mut self,
    ) -> Result<
        &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Allocator>, Allocator>,
        GraphicsError,
    > {
        let handle = match self.handle {
            Some(ref mut handle) => handle,
            None => {
                return Err(GraphicsError::InvalidCommandBufferUsage(
                    "tried to use command buffer that is not allocated",
                ))
            }
        };

        Ok(handle)
    }

    pub fn update_submitted(&mut self) {
        self.state = CommandBufferState::Submitted;
    }

    pub fn reset(&mut self) {
        self.state = CommandBufferState::Ready;
    }
}
