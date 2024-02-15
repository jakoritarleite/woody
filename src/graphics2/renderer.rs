use std::sync::Arc;

use thiserror::Error;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use super::vulkan::VulkanContext;

pub(crate) struct Renderer {
    pub(crate) window: Arc<Window>,
    vulkan: VulkanContext,
}

impl Renderer {
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title("Woody Engine")
            .build(event_loop)?;
        let window = Arc::new(window);

        let vulkan_context = VulkanContext::new(event_loop, window.clone())?;

        Ok(Self {
            window,
            vulkan: vulkan_context,
        })
    }
}

#[derive(Debug, Error)]
pub(crate) enum Error {
    /// Error that happens when creating a window.
    #[error("Could not create Window: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    #[error("The graphics backend failed: {0}")]
    Graphics(#[from] super::vulkan::Error),
}
