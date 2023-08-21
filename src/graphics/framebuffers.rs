use std::sync::Arc;

use vulkano::image::view::ImageView;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::Swapchain;

use super::context::Graphics;
use super::context::GraphicsError;

impl Graphics {
    /// Creates [Framebuffers](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFramebuffer.html) correspoding to the swapchain image views.
    pub fn create_framebuffers(
        swapchain: Arc<Swapchain>,
        swapchain_image_views: &[Arc<ImageView>],
        render_pass: Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>, GraphicsError> {
        let framebuffers = swapchain_image_views
            .iter()
            .map(|view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view.clone()],
                        extent: swapchain.image_extent(),
                        layers: 1,
                        ..Default::default()
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(framebuffers)
    }
}
