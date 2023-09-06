use std::sync::Arc;

use log::debug;
use log::info;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::device::Device;
use vulkano::format::ClearValue;
use vulkano::image::ImageLayout;
use vulkano::image::SampleCount;
use vulkano::render_pass::AttachmentDescription;
use vulkano::render_pass::AttachmentLoadOp;
use vulkano::render_pass::AttachmentReference;
use vulkano::render_pass::AttachmentStoreOp;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::RenderPass as vkRenderPass;
use vulkano::render_pass::RenderPassCreateInfo;
use vulkano::render_pass::SubpassDependency;
use vulkano::render_pass::SubpassDescription;
use vulkano::sync::AccessFlags;
use vulkano::sync::DependencyFlags;
use vulkano::sync::PipelineStages;

use crate::graphics::GraphicsError;

use super::swapchain::SwapchainContext;

pub enum RenderPassState {
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
}

pub struct RenderPass {
    handle: Arc<vkRenderPass>,
    state: RenderPassState,
    // x, y, w, h
    render_area: [u32; 4],
    // r, g, b, a
    clear_colors: [f32; 4],
    depth: f32,
    stencil: u32,
}

impl RenderPass {
    pub fn new(
        device: Arc<Device>,
        swapchain: &SwapchainContext,
        render_area: [u32; 4],
        clear_colors: [f32; 4],
        depth: f32,
        stencil: u32,
    ) -> Result<RenderPass, GraphicsError> {
        info!("Creating color attachment description");
        let color_attachment = AttachmentDescription {
            format: swapchain.image_format(),
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            stencil_load_op: Some(AttachmentLoadOp::DontCare),
            stencil_store_op: Some(AttachmentStoreOp::DontCare),
            ..Default::default()
        };

        info!("Creating color attachment reference");
        let color_attachment_reference = AttachmentReference {
            attachment: 0,
            layout: ImageLayout::ColorAttachmentOptimal,
            ..Default::default()
        };

        info!("Creating depth attachment description");
        let depth_attachment = AttachmentDescription {
            format: swapchain.depth_format,
            samples: SampleCount::Sample1,
            load_op: AttachmentLoadOp::Clear,
            store_op: AttachmentStoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            stencil_load_op: Some(AttachmentLoadOp::DontCare),
            stencil_store_op: Some(AttachmentStoreOp::DontCare),
            ..Default::default()
        };

        info!("Creating depth attachment reference");
        let depth_attachment_reference = AttachmentReference {
            attachment: 1,
            layout: ImageLayout::DepthStencilAttachmentOptimal,
            ..Default::default()
        };

        info!("Creating subpass description");
        // TODO: check to see how the VkSubpassDescription.pipelineBindPoint is set in vulkano.
        let subpass_description = SubpassDescription {
            color_attachments: vec![Some(color_attachment_reference)],
            depth_stencil_attachment: Some(depth_attachment_reference),
            ..Default::default()
        };

        info!("Creating subpass dependency");
        let subpass_dependency = SubpassDependency {
            // None specifies VK_SUBPASS_EXTERNAL.
            //
            // See: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_SUBPASS_EXTERNAL.html
            src_subpass: None,
            dst_subpass: Some(0),
            src_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
            dst_stages: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
            src_access: AccessFlags::empty(),
            dst_access: AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: DependencyFlags::empty(),
            ..Default::default()
        };

        let info = RenderPassCreateInfo {
            attachments: vec![color_attachment, depth_attachment],
            subpasses: vec![subpass_description],
            dependencies: vec![subpass_dependency],
            ..Default::default()
        };

        info!("Creating render_pass");
        let render_pass = vkRenderPass::new(device, info)?;

        Ok(Self {
            handle: render_pass,
            state: RenderPassState::Ready,
            render_area,
            clear_colors,
            depth,
            stencil,
        })
    }

    pub fn begin(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        frame_buffer: Arc<Framebuffer>,
    ) -> Result<(), GraphicsError> {
        let color_clear_value = ClearValue::Float(self.clear_colors);
        let depth_clear_value = ClearValue::DepthStencil((self.depth, self.stencil));

        let begin_info = RenderPassBeginInfo {
            render_area_offset: [self.render_area[0], self.render_area[1]],
            render_area_extent: [self.render_area[2], self.render_area[3]],
            clear_values: vec![Some(color_clear_value), Some(depth_clear_value)],
            ..RenderPassBeginInfo::framebuffer(frame_buffer)
        };

        command_buffer.begin_render_pass(begin_info, Default::default())?;

        Ok(())
    }

    pub fn end(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<(), GraphicsError> {
        command_buffer.end_render_pass(Default::default())?;

        Ok(())
    }
}
