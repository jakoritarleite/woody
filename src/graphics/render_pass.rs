use std::sync::Arc;

use vulkano::device::Device;
use vulkano::image::ImageLayout;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::Swapchain;

use super::vulkan::Graphics;
use super::GraphicsError;

impl Graphics {
    /// Creates a [Render Pass](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkRenderPass.html).
    pub fn create_render_pass(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Arc<RenderPass>, GraphicsError> {
        let render_pass = vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::PresentSrc,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {}
            },
        )?;

        Ok(render_pass)
    }
}
