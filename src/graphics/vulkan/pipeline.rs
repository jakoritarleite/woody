use std::sync::Arc;

use glam::Mat4;
use log::debug;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::device::Device;
use vulkano::image::SampleCount;
use vulkano::pipeline::graphics::color_blend::AttachmentBlend;
use vulkano::pipeline::graphics::color_blend::BlendFactor;
use vulkano::pipeline::graphics::color_blend::BlendOp;
use vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::color_blend::ColorComponents;
use vulkano::pipeline::graphics::depth_stencil::CompareOp;
use vulkano::pipeline::graphics::depth_stencil::DepthState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::depth_stencil::StencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
use vulkano::pipeline::graphics::rasterization::PolygonMode;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::layout::PushConstantRange;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::StateMode;
use vulkano::render_pass::Subpass;
use vulkano::shader::ShaderStages;

use crate::graphics::vertex::Vertex;
use crate::graphics::GraphicsError;

use super::command_buffer::CommandBuffer;
use super::renderpass::RenderPass;

pub struct Pipeline {
    handle: Arc<GraphicsPipeline>,
    pub layout: Arc<PipelineLayout>,
}

impl Pipeline {
    pub fn new(
        device: Arc<Device>,
        render_pass: &RenderPass,
        vertex_input_attribute_descriptions: Vec<(u32, VertexInputAttributeDescription)>,
        descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
        stages: Vec<PipelineShaderStageCreateInfo>,
        is_wireframe: bool,
    ) -> Result<Self, GraphicsError> {
        // TODO: make this configurable?
        let viewport_state = ViewportState::viewport_dynamic_scissor_irrelevant();

        let polygon_mode = if is_wireframe {
            PolygonMode::Line
        } else {
            PolygonMode::Fill
        };

        let rasterizer_state = RasterizationState {
            polygon_mode,
            line_width: StateMode::Fixed(1.0),
            cull_mode: StateMode::Fixed(CullMode::Back),
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            ..Default::default()
        };

        let multisample_state = MultisampleState {
            rasterization_samples: SampleCount::Sample1,
            sample_shading: Some(1.0),
            ..Default::default()
        };

        let depth_stencil_state = DepthStencilState {
            depth: Some(DepthState {
                write_enable: StateMode::Fixed(true),
                compare_op: StateMode::Fixed(CompareOp::Less),
                ..Default::default()
            }),
            ..Default::default()
        };

        let color_blend_attachment_state = ColorBlendAttachmentState {
            blend: Some(AttachmentBlend {
                src_color_blend_factor: BlendFactor::SrcAlpha,
                dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                color_blend_op: BlendOp::Add,
                src_alpha_blend_factor: BlendFactor::SrcAlpha,
                dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
                alpha_blend_op: BlendOp::Add,
            }),
            color_write_mask: ColorComponents::all(),
            color_write_enable: StateMode::Fixed(true),
        };

        let color_blend_state = ColorBlendState {
            logic_op: None,
            attachments: vec![color_blend_attachment_state],
            ..Default::default()
        };

        let (vertex_binding, vertex_description) = Vertex::binding_description();
        let vertex_input_state = VertexInputState::new()
            .binding(vertex_binding, vertex_description)
            .attributes(vertex_input_attribute_descriptions);

        let input_assembly_state =
            InputAssemblyState::new().topology(PrimitiveTopology::TriangleList);

        #[allow(dead_code)]
        struct PushConstant {
            model: Mat4,
        }

        let push_constant = PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: std::mem::size_of::<PushConstant>() as u32,
        };

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts: descriptor_set_layouts,
                push_constant_ranges: vec![push_constant],
                ..Default::default()
            },
        )?;

        let pipeline = GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(input_assembly_state),
                viewport_state: Some(viewport_state),
                rasterization_state: Some(rasterizer_state),
                multisample_state: Some(multisample_state),
                depth_stencil_state: Some(depth_stencil_state),
                color_blend_state: Some(color_blend_state),
                subpass: Subpass::from(render_pass.handle(), 0).map(|subpass| subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
            },
        )?;

        debug!("Graphics pipeline created");

        Ok(Self {
            handle: pipeline,
            layout: pipeline_layout,
        })
    }

    pub fn bind(&self, command_buffer: &mut CommandBuffer) -> Result<(), GraphicsError> {
        command_buffer
            .handle_mut()?
            .bind_pipeline_graphics(self.handle.clone())?;
        Ok(())
    }
}
