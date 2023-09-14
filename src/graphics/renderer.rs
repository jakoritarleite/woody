use std::sync::Arc;

use log::debug;
use log::error;
use log::trace;
use thiserror::Error;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

// use super::vulkan::Graphics;
use super::vulkan::VulkanContext;
use super::GraphicsError;

/// Renderer is a frontend that will be used by our systems.
pub(crate) struct Renderer {
    window: Arc<Window>,
    backend: VulkanContext,
}

impl Renderer {
    /// Create a new instance of [`Renderer`].
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Result<Self, RendererError> {
        let window = WindowBuilder::new()
            .with_title("Woody Engine")
            .build(event_loop)?;
        let window = Arc::new(window);

        let backend = VulkanContext::new(event_loop, window.clone())?;

        Ok(Self { window, backend })
    }

    /// TODO: document this.
    pub(crate) fn resize(&mut self) -> Result<(), RendererError> {
        self.backend.recreate_swapchain = true;

        Ok(())
    }

    /// TODO: document this.
    pub(crate) fn draw_frame(&mut self) -> Result<(), RendererError> {
        let result = self.backend.begin_frame();

        let finish_frame = match result {
            Ok(value) => value,
            err @ Err(_) => {
                error!("Backend begin frame error: {:?}", err);
                err?
            }
        };

        if finish_frame {
            self.backend.end_frame()?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    /// Error that happens when creating a window.
    #[error("Could not create Window: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    #[error("The graphics backend failed: {0}")]
    Graphics(#[from] GraphicsError),
}
