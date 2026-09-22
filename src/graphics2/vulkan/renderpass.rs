use std::sync::Arc;

use ash::vk;

use crate::graphics2::RenderArea;
use crate::graphics2::Rgba;

use super::command_buffer::CommandBuffer;
use super::device::Device;
use super::framebuffer::Framebuffer;
use super::swapchain::SwapchainContext;
use super::Error;

/// Vulkan abstraction of RenderPass.
pub struct RenderPass {
    pub handle: vk::RenderPass,
    pub render_area: RenderArea,
    pub clear_colors: Rgba,
    pub depth: f32,
    pub stencil: u32,
    _device: Arc<Device>,
}

impl RenderPass {
    /// Creates a new instance of [`RenderPass`].
    pub fn new(
        device: Arc<Device>,
        swapchain: &SwapchainContext,
        render_area: RenderArea,
        clear_colors: Rgba,
        depth: f32,
        stencil: u32,
    ) -> Result<Self, Error> {
        log::info!("Creating color attachment description");
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.image_format())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE);
        let color_attachment_reference = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        log::info!(
            "Creating depth attachment description with format: {:?}",
            swapchain.depth_format
        );
        let depth_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE);
        let depth_attachment_reference = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let attachments = [color_attachment.build(), depth_attachment.build()];

        log::info!("Creating subpass description");
        let subpass_description = vk::SubpassDescription::builder()
            .color_attachments(std::slice::from_ref(&color_attachment_reference))
            .depth_stencil_attachment(&depth_attachment_reference);

        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            )
            .dependency_flags(vk::DependencyFlags::empty());

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass_description))
            .dependencies(std::slice::from_ref(&subpass_dependency));

        log::info!("Creating renderpass");
        let renderpass = unsafe { device.create_render_pass(&renderpass_create_info, None)? };

        Ok(Self {
            handle: renderpass,
            render_area,
            clear_colors,
            depth,
            stencil,
            _device: device,
        })
    }

    pub fn begin(&self, command_buffer: &CommandBuffer, framebuffer: &Framebuffer) {
        let clear_color_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: Into::<[f32; 4]>::into(self.clear_colors),
            },
        };
        let clear_depth_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: self.depth,
                stencil: self.stencil,
            },
        };
        let clear_values = [clear_color_value, clear_depth_value];

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.handle)
            .render_area(self.render_area.into())
            .clear_values(&clear_values)
            .framebuffer(framebuffer.handle);

        unsafe {
            self._device.cmd_begin_render_pass(
                command_buffer.handle,
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub fn end(&self, command_buffer: &CommandBuffer) {
        unsafe {
            self._device.cmd_end_render_pass(command_buffer.handle);
        }
    }

    /// Returns a mutable reference to [`Self::render_area`].
    pub fn render_area_mut(&mut self) -> &mut RenderArea {
        &mut self.render_area
    }
}
