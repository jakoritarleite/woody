use std::ffi::CString;
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
    /// Kept alive so `stage_create_info.p_name` stays valid.
    entry_point: CString,
    stage_create_info: vk::PipelineShaderStageCreateInfo,
}

impl ShaderStage {
    pub fn new(
        module: ShaderModule,
        entry_point: impl Into<String>,
        stage: vk::ShaderStageFlags,
    ) -> Self {
        let entry_point =
            CString::new(entry_point.into()).expect("entry point name contains a NUL byte");

        let stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .module(module.handle)
            .name(entry_point.as_c_str())
            .stage(stage)
            .build();

        Self {
            handle: Arc::new(module),
            entry_point,
            stage_create_info,
        }
    }

    pub fn stage_create_info(&self) -> vk::PipelineShaderStageCreateInfo {
        self.stage_create_info
    }
}
