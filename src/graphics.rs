use thiserror::Error;

pub mod renderer;
pub mod uniform;
mod vertex;
pub mod vulkan;

#[derive(Debug, Error)]
pub enum GraphicsError {
    /// Error that happens when loading Vulkan library.
    #[error("Could not load Vulkan library: {0}")]
    LibraryLoading(#[from] vulkano::LoadingError),

    #[error("Could not fetch window or display handle: {0}")]
    Handle(#[from] raw_window_handle::HandleError),

    #[error("Could not create swapchain handle from window")]
    SwapchainFromWindow(#[from] vulkano::swapchain::FromWindowError),

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
    BufferAllocate(#[from] vulkano::Validated<vulkano::buffer::AllocateBufferError>),

    #[error("Could not allocate image: {0}")]
    ImageAllocate(#[from] vulkano::Validated<vulkano::image::AllocateImageError>),

    #[error("Could not read or write resource from CPU: {0}")]
    HostAccess(#[from] vulkano::sync::HostAccessError),

    #[error("Device does not support any candidate depth formats")]
    NoSupportedDepthFormat,

    #[error("Cannot perform command in a command_buffer that is in a invalid state: {0}")]
    InvalidCommandBufferUsage(&'static str),

    #[error("Could not allocate memory: {0}")]
    MemoryAllocator(#[from] vulkano::memory::allocator::MemoryAllocatorError),

    #[error("Unsupported layout transition from {0:?} to {1:?}")]
    LayoutTransition(vulkano::image::ImageLayout, vulkano::image::ImageLayout),
}
