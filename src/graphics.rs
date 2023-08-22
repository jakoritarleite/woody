use thiserror::Error;

pub mod context;
mod framebuffers;
pub mod mesh;
mod pipeline;
mod render_pass;
mod swapchain;
mod vertex;

#[derive(Debug, Error)]
pub enum GraphicsError {
    /// Error that happens when creating a window.
    #[error("Could not create Window: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    /// Error that happens when loading Vulkan library.
    #[error("Could not load Vulkan library: {0}")]
    LibraryLoading(#[from] vulkano::LoadingError),

    #[error("Could not validate: {0}")]
    VulkanValidation(#[from] vulkano::Validated<vulkano::VulkanError>),

    #[error("Vulkan runtime error: {0}")]
    Vulkan(#[from] vulkano::VulkanError),

    #[error("Could not find any suitable physical device")]
    NoSuitablePhysicalDevice,

    #[error("Could not create needed devices queues.")]
    NoDeviceQueues,

    #[error("Synchronization mechanism wasn't initialized due to an unknown reason.")]
    SynchronizationNotInitialized,

    #[error("Validation error: {0}")]
    Validation(#[from] Box<vulkano::ValidationError>),

    #[error("Error executing command buffer: {0}")]
    CommandBufferExecution(#[from] vulkano::command_buffer::CommandBufferExecError),

    #[error("Could not find shader entry point: {0}")]
    WrongShaderEntryPoint(&'static str),

    #[error("Could not allocate buffer: {0}")]
    BufferAllocate(#[from] vulkano::Validated<vulkano::buffer::BufferAllocateError>),

    #[error("Could not read or write resource from CPU: {0}")]
    HostAccess(#[from] vulkano::sync::HostAccessError),
}
