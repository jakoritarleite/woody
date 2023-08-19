use std::collections::BTreeMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use nalgebra_glm::vec2;
use nalgebra_glm::vec3;
use thiserror::Error;
use vulkano::buffer::Buffer;
use vulkano::buffer::BufferAllocateError;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::IndexBuffer;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::pool::CommandPool;
use vulkano::command_buffer::pool::CommandPoolCreateInfo;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferExecError;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::PrimaryCommandBufferAbstract;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::command_buffer::SemaphoreSubmitInfo;
use vulkano::command_buffer::SubmitInfo;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::SubpassEndInfo;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::layout::DescriptorSetLayoutBinding;
use vulkano::descriptor_set::layout::DescriptorSetLayoutCreateInfo;
use vulkano::descriptor_set::layout::DescriptorType;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceExtensions;
use vulkano::device::Queue;
use vulkano::device::QueueCreateInfo;
use vulkano::device::QueueFlags;
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::image::sampler::ComponentMapping;
use vulkano::image::sampler::ComponentSwizzle;
use vulkano::image::view::ImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::image::view::ImageViewType;
use vulkano::image::Image;
use vulkano::image::ImageLayout;
use vulkano::image::ImageSubresourceRange;
use vulkano::image::ImageUsage;
use vulkano::image::SampleCount;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
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
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::StateMode;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::shader::DescriptorBindingRequirements;
use vulkano::shader::ShaderStages;
use vulkano::swapchain::acquire_next_image;
use vulkano::swapchain::ColorSpace;
use vulkano::swapchain::CompositeAlpha;
use vulkano::swapchain::PresentInfo;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::swapchain::SwapchainPresentInfo;
use vulkano::sync;
use vulkano::sync::fence::Fence;
use vulkano::sync::fence::FenceCreateFlags;
use vulkano::sync::fence::FenceCreateInfo;
use vulkano::sync::semaphore::Semaphore;
use vulkano::sync::semaphore::SemaphoreCreateInfo;
use vulkano::sync::GpuFuture;
use vulkano::sync::HostAccessError;
use vulkano::sync::PipelineStage;
use vulkano::sync::Sharing;
use vulkano::DeviceSize;
use vulkano::LoadingError;
use vulkano::Validated;
use vulkano::ValidationError;
use vulkano::VulkanError;
use vulkano::VulkanLibrary;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use self::vertex::ColoredVertex;
use self::vertex_shader::UniformBufferObject;

mod vertex;

lazy_static! {
    pub(crate) static ref VERTICES: Vec<ColoredVertex> = vec![
        ColoredVertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0)),
        ColoredVertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0)),
        ColoredVertex::new(vec2(0.5, 0.5), vec3(0.0, 0.0, 1.0)),
        ColoredVertex::new(vec2(-0.5, 0.5), vec3(1.0, 1.0, 1.0)),
    ];
}

pub(crate) const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

#[allow(dead_code)]
pub struct Renderer {
    pub(crate) window: Arc<Window>,

    pub(crate) instance: Arc<Instance>,
    pub(crate) surface: Arc<Surface>,

    pub(crate) physical_device: Arc<PhysicalDevice>,
    pub(crate) device: Arc<Device>,

    pub(crate) queue: Arc<Queue>,

    pub(crate) viewport: Viewport,

    pub(crate) swapchain: Arc<Swapchain>,
    pub(crate) swapchain_images: Vec<Arc<Image>>,
    pub(crate) swapchain_image_views: Vec<Arc<ImageView>>,

    pub(crate) render_pass: Arc<RenderPass>,

    pub(crate) descriptor_set_layout: Arc<DescriptorSetLayout>,

    pub(crate) graphics_pipeline: Arc<GraphicsPipeline>,
    pub(crate) graphics_pipeline_layout: Arc<PipelineLayout>,

    pub(crate) framebuffers: Vec<Arc<Framebuffer>>,

    pub(crate) command_pool: CommandPool,

    pub(crate) vertex_subbuffer: Subbuffer<[ColoredVertex]>,
    pub(crate) index_buffer: IndexBuffer,

    pub(crate) uniform_buffers: Vec<Subbuffer<UniformBufferObject>>,
    pub(crate) uniform_buffer_sets: Vec<Arc<PersistentDescriptorSet>>,

    pub(crate) command_buffer_allocator: StandardCommandBufferAllocator,
    pub(crate) previous_frame_end: Option<Box<dyn GpuFuture>>,

    pub(crate) recreate_swapchain: bool,
    pub(crate) perspective_angle: f32,
}

impl Renderer {
    pub(crate) fn new(event_loop: &EventLoop<()>) -> Result<Self, RendererError> {
        let window = WindowBuilder::new()
            .with_title("Woody Engine")
            .with_inner_size(LogicalSize::new(1024, 768))
            .build(event_loop)
            .map_err(|err| RendererError::Window(err.to_string()))?;
        let window = Arc::new(window);

        let library = VulkanLibrary::new()?;
        let required_extensions = Surface::required_extensions(&event_loop);

        let info = InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..InstanceCreateInfo::application_from_cargo_toml()
        };

        let instance = Instance::new(library, info)?;
        let surface = Surface::from_window(instance.clone(), window.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::pick_physical_device(instance.clone(), surface.clone(), &device_extensions)?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,

                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    queues: vec![1.0],
                    ..Default::default()
                }],

                ..Default::default()
            },
        )?;
        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        let queue = queues.next().ok_or(RendererError::NoDeviceQueues)?;

        let (swapchain, swapchain_images, swapchain_image_views) = Self::create_swapchain(
            &window,
            physical_device.clone(),
            device.clone(),
            surface.clone(),
        )?;

        let render_pass = Self::create_render_pass(device.clone(), swapchain.clone())?;
        let descriptor_set_layout =
            Self::create_descriptor_set_layout(device.clone(), swapchain.clone())?;

        let (graphics_pipeline, graphics_pipeline_layout, mut viewport) = Self::create_pipeline(
            device.clone(),
            swapchain.clone(),
            descriptor_set_layout.clone(),
            render_pass.clone(),
        )?;

        let framebuffers = window_size_dependent_setup(
            &swapchain_images,
            &swapchain_image_views,
            render_pass.clone(),
            &mut viewport,
        )?;

        let command_pool_create_info = CommandPoolCreateInfo {
            queue_family_index,
            ..Default::default()
        };

        let command_pool = CommandPool::new(device.clone(), command_pool_create_info)?;

        let vertex_buffer = Buffer::from_iter(
            &memory_allocator,
            BufferCreateInfo {
                sharing: Sharing::Exclusive,
                usage: BufferUsage::TRANSFER_SRC | BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            VERTICES.clone().into_iter(),
        )?;

        let index_subbuffer = Buffer::new_slice(
            &memory_allocator,
            BufferCreateInfo {
                sharing: Sharing::Exclusive,
                usage: BufferUsage::TRANSFER_SRC | BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            INDICES.len() as DeviceSize,
        )?;
        index_subbuffer.write()?.copy_from_slice(INDICES);

        let index_buffer = IndexBuffer::from(index_subbuffer);

        // Create a pool of uniform buffers, one per frame in flight. This way we always have an
        // available buffer to write during each frame while reusing them as much as possible.
        let uniform_buffers = (0..swapchain.image_count())
            .map(|_| {
                Buffer::new_sized::<UniformBufferObject>(
                    &memory_allocator,
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let uniform_buffer_sets = uniform_buffers
            .iter()
            .map(|buffer| {
                PersistentDescriptorSet::new(
                    &descriptor_set_allocator,
                    graphics_pipeline_layout.set_layouts()[0].clone(),
                    [WriteDescriptorSet::buffer(0, buffer.clone())],
                    [],
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        // TODO check if we need descriptor_pool (apparently not)
        //let descriptor_pool = {
        //    let create_info = DescriptorPoolCreateInfo {
        //        max_sets: swapchain_images.len() as u32,
        //        pool_sizes: AHashMap::from([(
        //            DescriptorType::UniformBuffer,
        //            swapchain_images.len() as u32,
        //        )])
        //        .into(),
        //        ..Default::default()
        //    };

        //    DescriptorPool::new(device.clone(), create_info)?
        //};

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Ok(Self {
            window,
            instance,
            surface,
            physical_device,
            device,
            queue,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            viewport,
            render_pass,
            descriptor_set_layout,
            graphics_pipeline,
            graphics_pipeline_layout,
            framebuffers,
            command_pool,
            vertex_subbuffer: vertex_buffer,
            index_buffer,
            uniform_buffers,
            uniform_buffer_sets,
            command_buffer_allocator,
            previous_frame_end,
            recreate_swapchain: false,
            perspective_angle: 45.0,
        })
    }

    pub(crate) fn render(&mut self) -> Result<(), RendererError> {
        // Don't draw the frame when the screen size is zero.
        let image_extent: [u32; 2] = self.window.inner_size().into();

        if image_extent.contains(&0) {
            return Ok(());
        }

        // TODO find a better error message to this.
        self.previous_frame_end
            .as_mut()
            .ok_or(RendererError::Synchronization(
                "Previous Frame End".to_string(),
            ))?
            .cleanup_finished();

        if self.recreate_swapchain {
            self.recreate_swapchain(image_extent)?;
        }

        // acquire next image
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(next_image) => next_image,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(err) => return Err(RendererError::from(err)),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.update_uniform_buffer(image_index)?;

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0]))],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                vulkano::command_buffer::SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .bind_pipeline_graphics(self.graphics_pipeline.clone())?
            .bind_vertex_buffers(0, self.vertex_subbuffer.clone())?
            .bind_index_buffer(self.index_buffer.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.graphics_pipeline.layout().clone(),
                0,
                self.uniform_buffer_sets[image_index as usize].clone(),
            )?
            .draw_indexed(INDICES.len() as u32, 1, 0, 0, 0)?
            .end_render_pass(SubpassEndInfo::default())?;

        let command_buffer = builder.build()?;

        let future = self
            .previous_frame_end
            .take()
            .ok_or(RendererError::Synchronization(
                "Preview Frame End".to_string(),
            ))?
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }

            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }

            Err(err) => {
                return Err(RendererError::Synchronization(format!(
                    "Failed to flush future: {}",
                    err
                )))
            }
        }

        Ok(())
    }

    fn recreate_swapchain(&mut self, image_extent: [u32; 2]) -> Result<(), RendererError> {
        if image_extent.contains(&0) {
            return Ok(());
        }

        let (swapchain, images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_extent,
            ..self.swapchain.create_info()
        })?;

        self.swapchain = swapchain;
        self.swapchain_images = images;

        // TODO change the swapchain image views to a method
        self.swapchain_image_views = self
            .swapchain_images
            .iter()
            .map(|image| {
                let components = ComponentMapping {
                    r: ComponentSwizzle::Identity,
                    g: ComponentSwizzle::Identity,
                    b: ComponentSwizzle::Identity,
                    a: ComponentSwizzle::Identity,
                };

                let subresource_range =
                    ImageSubresourceRange::from_parameters(self.swapchain.image_format(), 1, 1);

                let create_info = ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format: self.swapchain.image_format(),
                    component_mapping: components,
                    subresource_range,
                    ..Default::default()
                };

                ImageView::new(image.clone(), create_info)
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.framebuffers = window_size_dependent_setup(
            &self.swapchain_images,
            &self.swapchain_image_views,
            self.render_pass.clone(),
            &mut self.viewport,
        )?;

        self.recreate_swapchain = false;

        Ok(())
    }

    fn pick_physical_device(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32), RendererError> {
        instance
            .enumerate_physical_devices()?
            .filter(|device| device.supported_extensions().contains(device_extensions))
            .filter_map(|device| {
                device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(index, queue)| {
                        queue.queue_flags.contains(QueueFlags::GRAPHICS)
                            && device
                                .surface_support(index as u32, &surface)
                                .unwrap_or(false)
                    })
                    .map(|index| (device, index as u32))
            })
            .min_by_key(|(device, _)| match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .ok_or(RendererError::NoSuitablePhysicalDevice)
    }

    #[allow(clippy::type_complexity)]
    fn create_swapchain(
        window: &Window,
        physical_device: Arc<PhysicalDevice>,
        device: Arc<Device>,
        surface: Arc<Surface>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>, Vec<Arc<ImageView>>), RendererError> {
        let surface_capabilities =
            physical_device.surface_capabilities(&surface, Default::default())?;
        let surface_formats = physical_device.surface_formats(&surface, Default::default())?;
        let mut surface_present_modes = physical_device.surface_present_modes(&surface)?;

        let present_mode = surface_present_modes
            .find(|mode| *mode == PresentMode::Mailbox)
            .unwrap_or(PresentMode::Fifo);

        let (image_format, image_color_space) = surface_formats
            .iter()
            .cloned()
            .find(|(format, color_space)| {
                *format == Format::B8G8R8A8_SRGB && *color_space == ColorSpace::SrgbNonLinear
            })
            .unwrap_or_else(|| surface_formats[0]);

        let image_count = if let Some(max) = surface_capabilities.max_image_count {
            match max {
                0 => surface_capabilities.min_image_count + 1,
                _ => max,
            }
        } else {
            2
        };

        let create_info = SwapchainCreateInfo {
            min_image_count: image_count,
            image_format,
            image_color_space,
            image_extent: window.inner_size().into(),
            image_array_layers: 1,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            image_sharing: Sharing::Exclusive,
            pre_transform: surface_capabilities.current_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode,
            clipped: true,
            ..Default::default()
        };

        let (swapchain, images) = Swapchain::new(device, surface, create_info)?;

        let image_views = images
            .iter()
            .map(|image| {
                let components = ComponentMapping {
                    r: ComponentSwizzle::Identity,
                    g: ComponentSwizzle::Identity,
                    b: ComponentSwizzle::Identity,
                    a: ComponentSwizzle::Identity,
                };

                let subresource_range = ImageSubresourceRange::from_parameters(image_format, 1, 1);

                let create_info = ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format: image_format,
                    component_mapping: components,
                    subresource_range,
                    ..Default::default()
                };

                ImageView::new(image.clone(), create_info)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((swapchain, images, image_views))
    }

    fn create_render_pass(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Arc<RenderPass>, RendererError> {
        let render_pass = vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                    initial_layout: ImageLayout::Undefined,
                    final_layout: ImageLayout::PresentSrc,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {}
            },
        )?;

        Ok(render_pass)
    }

    fn create_descriptor_set_layout(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Arc<DescriptorSetLayout>, RendererError> {
        let requirements = DescriptorBindingRequirements {
            descriptor_types: vec![DescriptorType::UniformBuffer],
            descriptor_count: Some(1),
            image_format: Some(swapchain.image_format()),
            stages: ShaderStages::VERTEX,
            ..Default::default()
        };

        let ubo_binding = DescriptorSetLayoutBinding::from(&requirements);

        let create_info = DescriptorSetLayoutCreateInfo {
            bindings: BTreeMap::from([(0, ubo_binding)]),
            ..Default::default()
        };

        let descriptor_set_layout = DescriptorSetLayout::new(device, create_info)?;

        Ok(descriptor_set_layout)
    }

    fn create_pipeline(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
        descriptor_set_layout: Arc<DescriptorSetLayout>,
        render_pass: Arc<RenderPass>,
    ) -> Result<(Arc<GraphicsPipeline>, Arc<PipelineLayout>, Viewport), RendererError> {
        let vert_shader_module = vertex_shader::load(device.clone())?;
        let frag_shader_module = fragment_shader::load(device.clone())?;

        let vert_shader_entry_point = vert_shader_module
            .entry_point("main")
            .ok_or(RendererError::WrongShaderEntryPoint("main"))?;
        let frag_sader_entry_point = frag_shader_module
            .entry_point("main")
            .ok_or(RendererError::WrongShaderEntryPoint("main"))?;

        let vert_stage = PipelineShaderStageCreateInfo::new(vert_shader_entry_point);
        let frag_stage = PipelineShaderStageCreateInfo::new(frag_sader_entry_point);

        // TODO check if its correct
        let binding_descriptions = vec![ColoredVertex::binding_description()];
        let attribute_descriptions = ColoredVertex::attribute_descriptions();

        let vertex_input_state = VertexInputState::new()
            .bindings(binding_descriptions)
            .attributes(attribute_descriptions);

        let input_assembly_state =
            InputAssemblyState::new().topology(PrimitiveTopology::TriangleList);

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

        let viewport_state =
            ViewportState::viewport_fixed_scissor_fixed(vec![(viewport.clone(), scissor)]);

        let rasterization_state = RasterizationState {
            depth_clamp_enable: false,
            rasterizer_discard_enable: StateMode::Fixed(false),
            polygon_mode: PolygonMode::Fill,
            cull_mode: StateMode::Fixed(CullMode::Back),
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            depth_bias: None,
            line_width: StateMode::Fixed(1.0),
            ..Default::default()
        };

        let multisample_state = MultisampleState {
            rasterization_samples: SampleCount::Sample1,
            sample_shading: None,
            ..Default::default()
        };

        let attachment_state = ColorBlendAttachmentState {
            blend: None,
            color_write_mask: ColorComponents::all(),
            color_write_enable: StateMode::Fixed(true),
        };

        let color_blend_state = ColorBlendState {
            logic_op: None, // Some(StateMode::Fixed(LogicOp::Copy)),
            attachments: vec![attachment_state],
            blend_constants: StateMode::Fixed([0.0, 0.0, 0.0, 0.0]),
            ..Default::default()
        };

        let layout_create_info = PipelineLayoutCreateInfo {
            set_layouts: vec![descriptor_set_layout],
            ..Default::default()
        };

        let pipeline_layout = PipelineLayout::new(device.clone(), layout_create_info)?;

        let create_info = GraphicsPipelineCreateInfo {
            stages: vec![vert_stage, frag_stage].into(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(input_assembly_state),
            viewport_state: Some(viewport_state),
            rasterization_state: Some(rasterization_state),
            multisample_state: Some(multisample_state),
            color_blend_state: Some(color_blend_state),
            layout: pipeline_layout.clone(),
            subpass: Subpass::from(render_pass, 0).map(|subpass| subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
        };

        let pipeline = GraphicsPipeline::new(device, None, create_info)?;

        Ok((pipeline, pipeline_layout, viewport))
    }

    fn update_uniform_buffer(&mut self, image_index: u32) -> Result<(), RendererError> {
        let model = nalgebra_glm::rotate(
            &nalgebra_glm::identity(),
            // nalgebra_glm::radians(&nalgebra_glm::vec1(45.0))[0],
            nalgebra_glm::radians(&nalgebra_glm::vec1(self.perspective_angle))[0],
            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
        );

        let view = nalgebra_glm::look_at(
            &nalgebra_glm::vec3(2.0, 2.0, 2.0),
            &nalgebra_glm::vec3(0.0, 0.0, 0.0),
            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
        );

        let mut proj = nalgebra_glm::perspective(
            self.swapchain.image_extent()[0] as f32 / self.swapchain.image_extent()[1] as f32,
            nalgebra_glm::radians(&nalgebra_glm::vec1(45.0))[0],
            0.1,
            10.0,
        );

        proj[(1, 1)] *= -1.0;

        let ubo = UniformBufferObject {
            model: model.into(),
            view: view.into(),
            proj: proj.into(),
        };

        *self.uniform_buffers[image_index as usize].write()? = ubo;

        Ok(())
    }
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Too lazy to implement this now")
    }
}

// Copied from: https://github.com/vulkano-rs/vulkano/blob/master/examples/src/bin/triangle.rs
/// This function is called once during initialization, then again whenever the window is resized.
fn window_size_dependent_setup(
    images: &[Arc<Image>],
    image_views: &[Arc<ImageView>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Result<Vec<Arc<Framebuffer>>, RendererError> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    let framebuffers = image_views
        .iter()
        .map(|view| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view.clone()],
                    ..Default::default()
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/shader.vert",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/shader.frag",
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Could not create window: {0}")]
    Window(String),

    #[error("Validation error: {0}")]
    Validation(#[from] Box<ValidationError>),

    #[error("Error validating: {0}")]
    VulkanValidation(#[from] Validated<VulkanError>),

    #[error("Vulkan returned error: {0}")]
    VulkanError(#[from] VulkanError),

    #[error("Could not load Vulkan Library: {0}")]
    LibraryLoading(#[from] LoadingError),

    #[error("Could not find any suitable physical device")]
    NoSuitablePhysicalDevice,

    #[error("Could not get needed device queues.")]
    NoDeviceQueues,

    #[error("Shader bytecode is not properly aligned.")]
    MisalignedShaderBytecode,

    #[error("Could not find shader entry point: {0}")]
    WrongShaderEntryPoint(&'static str),

    #[error("Error validating buffer: {0}")]
    BufferAllocationValidation(#[from] Validated<BufferAllocateError>),

    #[error("Error executing command buffer: {0}")]
    CommandBufferExecution(#[from] CommandBufferExecError),

    #[error("Error executing action in host: {0}")]
    HostAccess(#[from] HostAccessError),

    #[error("Could not synchronize: {0}")]
    Synchronization(String),
}
