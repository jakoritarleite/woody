use std::sync::Arc;

use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription;

use crate::graphics::vulkan::command_buffer::CommandBuffer;
use crate::graphics::vulkan::pipeline::Pipeline;
use crate::graphics::vulkan::renderpass::RenderPass;
use crate::graphics::vulkan::VulkanContext;
use crate::graphics::GraphicsError;

use super::ShaderStage;

const SHADER_STAGE_COUNT: usize = 2;

pub struct ObjectShader {
    stages: [ShaderStage; SHADER_STAGE_COUNT],
    pipeline: Pipeline,
}

impl ObjectShader {
    pub fn new(device: Arc<Device>, render_pass: &RenderPass) -> Result<Self, GraphicsError> {
        let vert_module = vertex_shader::load(device.clone())?;
        let frag_module = fragment_shader::load(device.clone())?;

        let stages = [
            ShaderStage::new(vert_module, "main")?,
            ShaderStage::new(frag_module, "main")?,
        ];

        let pipeline_stages = stages
            .iter()
            .map(|shader| shader.pipeline_stage_create_info.clone())
            .collect();

        // TODO: descriptors

        let attribute_descriptions = vec![(
            0,
            VertexInputAttributeDescription {
                binding: 0,
                format: Format::R32G32B32_SFLOAT,
                offset: 0,
            },
        )];

        let pipeline = Pipeline::new(
            device,
            render_pass,
            attribute_descriptions,
            vec![],
            pipeline_stages,
            false,
        )?;

        Ok(Self { stages, pipeline })
    }

    pub fn bind(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), crate::graphics::GraphicsError> {
        self.pipeline.bind(command_buffer)?;

        Ok(())
    }
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/object/shader.vert",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/object/shader.frag",
    }
}
