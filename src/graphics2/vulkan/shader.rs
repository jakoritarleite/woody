use std::sync::Arc;

use ash::vk;

use super::device::Device;
use super::Error;

pub mod object;

pub struct ShaderModule {
    handle: vk::ShaderModule,
    create_info: vk::ShaderModuleCreateInfo,
}

impl ShaderModule {
    pub unsafe fn new(
        device: Arc<Device>,
        create_info: vk::ShaderModuleCreateInfo,
    ) -> Result<Self, Error> {
        let handle = device.create_shader_module(&create_info, None)?;

        Ok(Self {
            handle,
            create_info,
        })
    }
}

pub struct ShaderStage {
    handle: Arc<ShaderModule>,
    stage_create_info: vk::PipelineShaderStageCreateInfo,
}

impl ShaderStage {
    pub fn new(
        module: ShaderModule,
        entry_point: impl Into<String>,
        stage: vk::ShaderStageFlags,
    ) -> Self {
        let entry_point: String = entry_point.into();

        let stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .module(module.handle)
            .name(
                std::ffi::CString::new(entry_point.as_str())
                    .unwrap() // TODO: handle this error
                    .as_c_str(),
            )
            .stage(stage)
            .build();

        Self {
            handle: Arc::new(module),
            stage_create_info,
        }
    }
}
