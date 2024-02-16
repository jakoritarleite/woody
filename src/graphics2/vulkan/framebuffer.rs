use ash::vk;
use ash::Device;

use super::renderpass::RenderPass;
use super::swapchain::SwapchainContext;
use super::Error;

/// Vulkan abstraction to Framebuffer.
pub struct Framebuffer {
    pub handle: vk::Framebuffer,
}

impl Framebuffer {
    /// Creates a new instance of [`Framebuffer`].
    pub fn new(
        device: &Device,
        renderpass: &RenderPass,
        extent: [u32; 2],
        attachments: &[vk::ImageView],
    ) -> Result<Self, Error> {
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass.handle)
            .attachments(attachments)
            .width(extent[0])
            .height(extent[1])
            .layers(1);

        let framebuffer = unsafe { device.create_framebuffer(&create_info, None)? };

        Ok(Self {
            handle: framebuffer,
        })
    }
}

pub fn generate_framebuffers(
    device: &Device,
    renderpass: &RenderPass,
    swapchain: &SwapchainContext,
) -> Result<Vec<Framebuffer>, Error> {
    log::debug!(
        "Creating framebuffers for {} image views",
        swapchain.image_views.len()
    );

    swapchain
        .image_views
        .iter()
        .map(|image_view| {
            Framebuffer::new(
                device,
                renderpass,
                swapchain.extent,
                &[*image_view, swapchain.depth_attachment.view],
            )
        })
        .collect::<Result<Vec<_>, _>>()
}
