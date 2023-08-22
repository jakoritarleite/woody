use std::sync::Arc;

use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::device;
use vulkano::device::Device;
use vulkano::image::SampleCount;
use vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::color_blend::ColorComponents;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::rasterization::FrontFace;
use vulkano::pipeline::graphics::rasterization::PolygonMode;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::Scissor;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::StateMode;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::shader::ShaderModule;
use vulkano::swapchain::Swapchain;

use super::context::Graphics;
use super::vertex::Vertex;
use super::GraphicsError;

impl Graphics {
    /// Creates a [PipelineShaderStageCreateInfo](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineShaderStageCreateInfo.html) for a specific shader module.
    pub fn create_pipeline_shader_stage_create_info(
        shader: Arc<ShaderModule>,
    ) -> Result<PipelineShaderStageCreateInfo, GraphicsError> {
        Ok(PipelineShaderStageCreateInfo::new(
            shader
                .entry_point("main")
                .ok_or(GraphicsError::WrongShaderEntryPoint("main"))?,
        ))
    }

    /// Creates a VertexInputState.
    pub fn create_pipeline_vertex_input_state() -> VertexInputState {
        VertexInputState::new()
        //.bindings(vec![Vertex::binding_description()])
        //.attributes(Vertex::attribute_descriptions())
    }

    /// Creates a InputAssemblyState.
    pub fn create_pipeline_input_assembly_state(topology: PrimitiveTopology) -> InputAssemblyState {
        InputAssemblyState::new().topology(topology)
    }

    /// Creates a RasterizationState.
    pub fn create_pipeline_rasterization_state(polygon_mode: PolygonMode) -> RasterizationState {
        RasterizationState {
            depth_clamp_enable: false,
            rasterizer_discard_enable: StateMode::Fixed(false),
            polygon_mode,
            cull_mode: StateMode::Fixed(CullMode::None),
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            depth_bias: None,
            line_width: StateMode::Fixed(1.0),
            ..Default::default()
        }
    }

    /// Creates a MultisampleState.
    pub fn create_pipeline_multisampling_state() -> MultisampleState {
        MultisampleState {
            rasterization_samples: SampleCount::Sample1,
            sample_shading: None,
            alpha_to_one_enable: false,
            alpha_to_coverage_enable: false,
            ..Default::default()
        }
    }

    /// Creates a ColorBlendAttachmentState.
    pub fn create_pipeline_color_blend_attachment_state() -> ColorBlendAttachmentState {
        ColorBlendAttachmentState {
            blend: None,
            color_write_mask: ColorComponents::all(),
            color_write_enable: StateMode::Fixed(true),
        }
    }

    /// Creates a ColorBlendstate.
    pub fn create_pipeline_color_blend_state(
        attachment_state: ColorBlendAttachmentState,
    ) -> ColorBlendState {
        ColorBlendState {
            logic_op: None,
            attachments: vec![attachment_state],
            ..Default::default()
        }
    }

    fn create_descriptor_set_layout() -> Result<Arc<DescriptorSetLayout>, GraphicsError> {
        todo!()
    }

    pub fn create_triangle_pipeline(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
        render_pass: Arc<RenderPass>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
    ) -> Result<(Arc<GraphicsPipeline>, Arc<PipelineLayout>), GraphicsError> {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [
                swapchain.image_extent()[0] as f32,
                swapchain.image_extent()[1] as f32,
            ],
            depth_range: 0f32..=1f32,
        };

        let scissor = Scissor {
            offset: [0, 0],
            extent: swapchain.image_extent(),
        };

        let shader_stages = vec![
            Self::create_pipeline_shader_stage_create_info(vertex_shader)?,
            Self::create_pipeline_shader_stage_create_info(fragment_shader)?,
        ];

        let vertex_input_state = Self::create_pipeline_vertex_input_state();
        let input_assembly_state =
            Self::create_pipeline_input_assembly_state(PrimitiveTopology::TriangleList);
        let viewport_state = ViewportState::viewport_fixed_scissor_fixed(vec![(viewport, scissor)]);
        let rasterization_state = Self::create_pipeline_rasterization_state(PolygonMode::Fill);
        let multisampling_state = Self::create_pipeline_multisampling_state();
        let color_blend_attachment_state = Self::create_pipeline_color_blend_attachment_state();
        let color_blend_state =
            Self::create_pipeline_color_blend_state(color_blend_attachment_state);

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                // TODO create descriptor set for our triangle pipeline.
                // set_layouts: vec![Self::create_descriptor_set_layout()]
                ..Default::default()
            },
        )?;

        let pipeline = GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: shader_stages.into(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(input_assembly_state),
                viewport_state: Some(viewport_state),
                rasterization_state: Some(rasterization_state),
                multisample_state: Some(multisampling_state),
                color_blend_state: Some(color_blend_state),
                subpass: Subpass::from(render_pass, 0).map(|subpass| subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
            },
        )?;

        Ok((pipeline, pipeline_layout))
    }
}
