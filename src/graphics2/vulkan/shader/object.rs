use std::sync::Arc;

use ash::vk;
use glam::Mat4;

use crate::graphics2::vulkan::buffer::Buffer;
use crate::graphics2::vulkan::command_buffer::CommandBuffer;
use crate::graphics2::vulkan::device::Device;
use crate::graphics2::vulkan::pipeline::Pipeline;
use crate::graphics2::vulkan::renderpass::RenderPass;
use crate::graphics2::vulkan::uniform::GlobalUniformObject;

use super::Error;
use super::ShaderStage;

const SHADER_STAGE_COUNT: usize = 2;

pub struct ObjectShader {
    _stages: [ShaderStage; SHADER_STAGE_COUNT],
    _global_descriptor_set_layout: vk::DescriptorSetLayout,
    global_descriptor_sets: Vec<vk::DescriptorSet>,
    global_uniform_object: GlobalUniformObject,
    global_uniform_buffers: Vec<Buffer<GlobalUniformObject>>,
    pipeline: Pipeline,
    descriptor_pool: vk::DescriptorPool,
    device: Arc<Device>,
}

impl ObjectShader {
    pub fn new(
        device: Arc<Device>,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        render_pass: &RenderPass,
        swapchain_image_count: u32,
    ) -> Result<Self, Error> {
        let vert_module = vertex_shader::load(device.clone())?;
        let frag_module = fragment_shader::load(device.clone())?;

        let stages = [
            ShaderStage::new(vert_module, "main", vk::ShaderStageFlags::VERTEX),
            ShaderStage::new(frag_module, "main", vk::ShaderStageFlags::FRAGMENT),
        ];

        let pipeline_stages = [stages[0].stage_create_info(), stages[1].stage_create_info()];

        log::info!("Creating global uniform descriptor set layout");
        let ubo_binding = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];
        let layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_binding);
        let global_descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&layout_create_info, None)? };

        // Vertex layout: single binding of `position: Vec3`.
        let vertex_binding_description = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<glam::Vec3>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();
        let vertex_attribute_descriptions = [vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build()];

        let set_layouts = [global_descriptor_set_layout];
        let pipeline = Pipeline::new(
            device.clone(),
            render_pass,
            vertex_binding_description,
            &vertex_attribute_descriptions,
            &set_layouts,
            &pipeline_stages,
            false,
        )?;

        let mem_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let global_uniform_buffers = (0..swapchain_image_count)
            .map(|_| {
                Buffer::<GlobalUniformObject>::new(
                    device.clone(),
                    mem_properties,
                    1,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    mem_flags,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        log::info!("Allocating global descriptor sets");
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_image_count)
            .build()];
        let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(swapchain_image_count)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_create_info, None)? };

        let set_layouts_per_set =
            vec![global_descriptor_set_layout; swapchain_image_count as usize];
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&set_layouts_per_set);
        let global_descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

        let buffer_infos = global_uniform_buffers
            .iter()
            .map(|buffer| {
                vk::DescriptorBufferInfo::builder()
                    .buffer(buffer.handle)
                    .offset(0)
                    .range(std::mem::size_of::<GlobalUniformObject>() as u64)
                    .build()
            })
            .collect::<Vec<_>>();
        let writes = global_descriptor_sets
            .iter()
            .zip(&buffer_infos)
            .map(|(set, buffer_info)| {
                vk::WriteDescriptorSet::builder()
                    .dst_set(*set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(buffer_info))
                    .build()
            })
            .collect::<Vec<_>>();
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        Ok(Self {
            _stages: stages,
            _global_descriptor_set_layout: global_descriptor_set_layout,
            global_descriptor_sets,
            global_uniform_object: GlobalUniformObject::default(),
            global_uniform_buffers,
            pipeline,
            descriptor_pool,
            device,
        })
    }

    pub fn global_uniform_object_mut(&mut self) -> &mut GlobalUniformObject {
        &mut self.global_uniform_object
    }

    pub fn bind(&self, command_buffer: &CommandBuffer) {
        self.pipeline.bind(command_buffer);
    }

    /// Binds the per-image global descriptor set and uploads the current
    /// global uniform state.
    ///
    /// Writing to the uniform buffer must happen after waiting for the acquire
    /// future and before submitting the command buffer.
    pub fn update_global_state(
        &self,
        image_index: u32,
        command_buffer: &CommandBuffer,
    ) -> Result<(), Error> {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer.handle,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                std::slice::from_ref(&self.global_descriptor_sets[image_index as usize]),
                &[],
            );
        }

        self.global_uniform_buffers[image_index as usize]
            .write(std::slice::from_ref(&self.global_uniform_object))?;

        Ok(())
    }

    /// Records the model matrix into push constants.
    pub fn update_state(&self, model: Mat4, command_buffer: &CommandBuffer) {
        unsafe {
            self.device.cmd_push_constants(
                command_buffer.handle,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::bytes_of(&model),
            );
        }
    }
}

impl Drop for ObjectShader {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self._global_descriptor_set_layout, None);
        }
    }
}

mod vertex_shader {
    shader_macros::shader! {
        ty: "vertex",
        path: "src/shaders/object/shader.vert",
    }
}

mod fragment_shader {
    shader_macros::shader! {
        ty: "fragment",
        path: "src/shaders/object/shader.frag",
    }
}
