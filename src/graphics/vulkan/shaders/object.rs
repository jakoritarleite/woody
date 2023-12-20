use std::collections::BTreeMap;
use std::sync::Arc;

use glam::Mat4;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::layout::DescriptorSetLayoutBinding;
use vulkano::descriptor_set::layout::DescriptorSetLayoutCreateInfo;
use vulkano::descriptor_set::layout::DescriptorType;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::vertex_input::VertexInputAttributeDescription;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::shader::ShaderStages;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::sync::GpuFuture;

use crate::graphics::uniform::GlobalUniformObject;
use crate::graphics::uniform::UniformBuffer;
use crate::graphics::vulkan::command_buffer::CommandBuffer;
use crate::graphics::vulkan::pipeline::Pipeline;
use crate::graphics::vulkan::renderpass::RenderPass;
use crate::graphics::GraphicsError;

use super::ShaderStage;

const SHADER_STAGE_COUNT: usize = 2;

pub struct ObjectShader {
    stages: [ShaderStage; SHADER_STAGE_COUNT],
    global_descriptor_set_layout: Arc<DescriptorSetLayout>,
    global_descriptor_sets: Vec<Arc<DescriptorSet>>,
    global_uniform_object: GlobalUniformObject,
    global_uniform_buffers: Vec<UniformBuffer<GlobalUniformObject>>,
    pipeline: Pipeline,
}

impl ObjectShader {
    pub fn new(
        standard_allocator: Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        device: Arc<Device>,
        render_pass: &RenderPass,
        swapchain_image_count: u32,
    ) -> Result<Self, GraphicsError> {
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

        let global_ubo_descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo {
            bindings: BTreeMap::from([(
                0,
                DescriptorSetLayoutBinding {
                    stages: ShaderStages::VERTEX,
                    ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                },
            )]),
            ..Default::default()
        };

        let global_ubo_descriptor_set_layout =
            DescriptorSetLayout::new(device.clone(), global_ubo_descriptor_set_layout_create_info)?;

        let attribute_descriptions = vec![(
            0,
            VertexInputAttributeDescription {
                binding: 0,
                format: Format::R32G32B32_SFLOAT,
                offset: 0,
                ..Default::default()
            },
        )];

        let pipeline = Pipeline::new(
            device,
            render_pass,
            attribute_descriptions,
            vec![global_ubo_descriptor_set_layout.clone()],
            pipeline_stages,
            false,
        )?;

        let mut global_uniform_buffers = vec![];
        for _ in 0..=swapchain_image_count {
            global_uniform_buffers.push(UniformBuffer::<GlobalUniformObject>::new(
                standard_allocator.clone(),
            )?);
        }

        let global_descriptor_sets = global_uniform_buffers
            .iter()
            .map(|buffer| {
                DescriptorSet::new(
                    descriptor_set_allocator.clone(),
                    global_ubo_descriptor_set_layout.clone(),
                    vec![WriteDescriptorSet::buffer(0, buffer.handle())],
                    vec![],
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            stages,
            pipeline,
            global_descriptor_set_layout: global_ubo_descriptor_set_layout,
            global_descriptor_sets,
            global_uniform_object: GlobalUniformObject::default(),
            global_uniform_buffers,
        })
    }

    pub fn global_uniform_object_mut(&mut self) -> &mut GlobalUniformObject {
        &mut self.global_uniform_object
    }

    pub fn bind(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), crate::graphics::GraphicsError> {
        self.pipeline.bind(command_buffer)?;

        Ok(())
    }

    pub fn update_global_state(
        &mut self,
        image_index: u32,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), GraphicsError> {
        let descriptor_set = self.global_descriptor_sets[image_index as usize].clone();

        command_buffer.handle_mut()?.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline.layout.clone(),
            0,
            descriptor_set,
        )?;

        self.global_uniform_buffers[image_index as usize].load_data(self.global_uniform_object)?;

        Ok(())
    }

    pub fn update_state(
        &mut self,
        model: Mat4,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), GraphicsError> {
        command_buffer
            .handle_mut()?
            .push_constants(self.pipeline.layout.clone(), 0, model)?;

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
