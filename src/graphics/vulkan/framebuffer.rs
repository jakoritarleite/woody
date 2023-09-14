use std::sync::Arc;

use log::debug;
use vulkano::image::view::ImageView;
use vulkano::render_pass::Framebuffer as vkFramebuffer;
use vulkano::render_pass::FramebufferCreateInfo;

use crate::graphics::GraphicsError;

use super::renderpass::RenderPass;
use super::swapchain::SwapchainContext;

pub struct Framebuffer {
    handle: Arc<vkFramebuffer>,
}

impl Framebuffer {
    pub fn new(
        render_pass: &RenderPass,
        extent: [u32; 2],
        attachments: Vec<Arc<ImageView>>,
    ) -> Result<Self, GraphicsError> {
        let frambuffer = vkFramebuffer::new(
            render_pass.handle(),
            FramebufferCreateInfo {
                attachments,
                extent,
                layers: 1,
                ..Default::default()
            },
        )?;

        Ok(Self { handle: frambuffer })
    }

    pub fn handle(&self) -> Arc<vkFramebuffer> {
        self.handle.clone()
    }
}

pub fn generate_framebuffers(
    render_pass: &RenderPass,
    swapchain: &SwapchainContext,
) -> Result<Vec<Framebuffer>, GraphicsError> {
    let extent = [
        swapchain.image_width() as u32,
        swapchain.image_height() as u32,
    ];

    let depth = swapchain.depth_attachment.view.clone();

    dbg!(extent, depth.image().extent());

    let mut framebuffers = Vec::with_capacity(swapchain.image_views.len());
    for image_view in swapchain.image_views.iter() {
        let framebuffer = Framebuffer::new(
            render_pass,
            extent,
            vec![image_view.clone(), swapchain.depth_attachment.view.clone()],
        )?;

        framebuffers.push(framebuffer);
    }

    debug!("Created framebuffers");

    Ok(framebuffers)
}
