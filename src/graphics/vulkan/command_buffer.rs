use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

use crate::graphics::GraphicsError;

pub enum CommandBufferState {
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
    NotAllocated,
}
pub struct CommandBuffer {
    pub(super) handle: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    pub(super) state: CommandBufferState,
}

impl CommandBuffer {
    pub fn new(allocator: &impl CommandBufferAllocator) -> Result<Self, GraphicsError> {
        todo!();
    }
}
