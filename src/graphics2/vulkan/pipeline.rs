use std::sync::Arc;

use ash::vk;
use glam::Mat4;

use super::command_buffer::CommandBuffer;
use super::device::Device;
use super::renderpass::RenderPass;
use super::Error;

/// Abstraction of the Vulkan graphics pipeline.
pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    _device: Arc<Device>,
}

impl Pipeline {
    /// Creates a new instance of [`Pipeline`].
    pub fn new(
        device: Arc<Device>,
        render_pass: &RenderPass,
        vertex_binding_description: vk::VertexInputBindingDescription,
        vertex_attribute_descriptions: &[vk::VertexInputAttributeDescription],
        set_layouts: &[vk::DescriptorSetLayout],
        stages: &[vk::PipelineShaderStageCreateInfo],
        is_wireframe: bool,
    ) -> Result<Self, Error> {
        let polygon_mode = if is_wireframe {
            vk::PolygonMode::LINE
        } else {
            vk::PolygonMode::FILL
        };

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding_description))
            .vertex_attribute_descriptions(vertex_attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // Viewport and scissor are dynamic states, recorded at draw time.
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(polygon_mode)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(true)
            .min_sample_shading(1.0);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment_state));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<Mat4>() as u32);

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        log::debug!("Creating pipeline layout");
        let layout = unsafe { device.create_pipeline_layout(&layout_create_info, None)? };

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .render_pass(render_pass.handle)
            .subpass(0)
            .build();

        log::debug!("Creating graphics pipeline");
        let handle = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
        }
        .map_err(|(_, result)| result)?[0];

        Ok(Self {
            handle,
            layout,
            _device: device,
        })
    }

    /// Binds this pipeline as the current graphics pipeline.
    pub fn bind(&self, command_buffer: &CommandBuffer) {
        unsafe {
            self._device.cmd_bind_pipeline(
                command_buffer.handle,
                vk::PipelineBindPoint::GRAPHICS,
                self.handle,
            );
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self._device.destroy_pipeline(self.handle, None);
            self._device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
