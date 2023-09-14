//! # Shaders

use std::sync::Arc;

use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::shader::ShaderModule;

use crate::graphics::GraphicsError;

pub mod object;

pub struct ShaderStage {
    handle: Arc<ShaderModule>,
    pub(super) pipeline_stage_create_info: PipelineShaderStageCreateInfo,
}

impl ShaderStage {
    pub fn new(
        module: Arc<ShaderModule>,
        entry_point: &'static str,
    ) -> Result<Self, GraphicsError> {
        let pipeline_stage_create_info = PipelineShaderStageCreateInfo::new(
            module
                .entry_point(entry_point)
                .ok_or(GraphicsError::WrongShaderEntryPoint(entry_point))?,
        );

        Ok(Self {
            handle: module,
            pipeline_stage_create_info,
        })
    }
}
