use thiserror::Error;
use winit::event_loop::EventLoop;

use super::vulkan::Graphics;
use super::GraphicsError;

/// Renderer is a frontend that will be used by our systems.
#[derive(Debug)]
pub(crate) struct Renderer {
    backend: Graphics,
}

impl Renderer {
    /// Create a new instance of [`Renderer`].
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Result<Self, RendererError> {
        Ok(Self {
            backend: Graphics::new(event_loop)?,
        })
    }

    /// TODO: document this.
    pub(crate) fn resize(&mut self, width: f32, height: f32) -> Result<(), RendererError> {
        self.backend.recreate_swapchain = true;

        Ok(())
    }

    /// TODO: document this.
    pub(crate) fn draw_frame(&mut self) -> Result<(), RendererError> {
        self.backend.begin_frame()?;

        self.backend.end_frame()?;
        self.backend.frame_number += 1;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("The graphics backend failed: {0}")]
    Graphics(#[from] GraphicsError),
}
