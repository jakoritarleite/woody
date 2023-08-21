use std::fmt::Debug;
use std::sync::Arc;

use thiserror::Error;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::SubpassEndInfo;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceExtensions;
use vulkano::device::Queue;
use vulkano::device::QueueCreateInfo;
use vulkano::device::QueueFlags;
use vulkano::format::ClearValue;
use vulkano::image::view::ImageView;
use vulkano::image::Image;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::RenderPass;
use vulkano::shader::ShaderModule;
use vulkano::swapchain::acquire_next_image;
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainPresentInfo;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::Validated;
use vulkano::VulkanError;
use vulkano::VulkanLibrary;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

/// Vulkan graphics context.
#[derive(Debug)]
pub struct VulkanContext {
    /// Vulkan Instance.
    pub instance: Arc<Instance>,
    /// Vulkan swapchain screen Surface.
    pub surface: Arc<Surface>,
    /// Vulkan logical device.
    pub device: Arc<Device>,
    /// Vulkan graphics queue.
    pub queue: Arc<Queue>,
    /// Vulkan memory allocator.
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    /// Vulkan command buffer allocator.
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    /// Vulkan descriptor set allocator.
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl VulkanContext {
    pub fn new(event_loop: &EventLoop<()>, window: Arc<Window>) -> Result<Self, GraphicsError> {
        let library = VulkanLibrary::new()?;
        let required_extensions = Surface::required_extensions(event_loop);

        let info = InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..InstanceCreateInfo::application_from_cargo_toml()
        };
        let instance = Instance::new(library, info)?;

        let surface = Surface::from_window(instance.clone(), window)?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|device| device.supported_extensions().contains(&device_extensions))
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
            .ok_or(GraphicsError::NoSuitablePhysicalDevice)?;

        println!(
            "Selected physical device ( {} : {:?} )",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        let (device, mut queues) = Device::new(
            physical_device,
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
        let queue = queues.next().ok_or(GraphicsError::NoDeviceQueues)?;

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device.clone()));

        Ok(VulkanContext {
            instance,
            surface,
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        })
    }
}

/// A graphics context for Vulkan rendering.
#[allow(dead_code)]
pub struct Graphics {
    pub(crate) vulkan: VulkanContext,

    /// winit Window.
    pub(crate) window: Arc<Window>,

    /// How many frames we've drew.
    pub(crate) frame_number: usize,

    /// Vulkan swapchain.
    pub(super) swapchain: Arc<Swapchain>,
    pub(super) swapchain_images: Vec<Arc<Image>>,
    pub(super) swapchain_image_views: Vec<Arc<ImageView>>,
    /// Determines if the swapchain must be recreated.
    ///
    /// This is used when the window size changes.
    pub(crate) recreate_swapchain: bool,

    /// Command buffer allocator.
    ///
    /// This abstracts the usage of [Command Pools](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandPool.html).
    pub(super) command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    /// Render pass.
    pub(super) render_pass: Arc<RenderPass>,
    pub(super) framebuffers: Vec<Arc<Framebuffer>>,

    /// Shaders for our triangle geometries.
    ///
    /// For now all geometries uses this shaders, but in future we may have different shaders for
    /// circles and other things.
    triangle_vertex_shader: Arc<ShaderModule>,
    triangle_fragment_shader: Arc<ShaderModule>,

    /// Graphics Pipelines.
    triangle_pipeline: Arc<GraphicsPipeline>,
    triangle_pipeline_layout: Arc<PipelineLayout>,

    /// Vulkano Synchronization mechanism.
    sync: Option<Box<dyn GpuFuture>>,
}

impl Graphics {
    /// Creates a new [`Graphics`] from an event loop.
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, GraphicsError> {
        let window = WindowBuilder::new()
            .with_title("Woody Engine")
            .with_inner_size(LogicalSize::new(1024, 768))
            .build(event_loop)?;
        let window = Arc::new(window);

        let vulkan_ctx = VulkanContext::new(event_loop, window.clone())?;

        let (swapchain, swapchain_images) = Self::create_swapchain(
            window.clone(),
            vulkan_ctx.device.clone(),
            vulkan_ctx.surface.clone(),
        )?;
        let swapchain_image_views =
            Self::create_swapchain_image_views(swapchain.clone(), &swapchain_images)?;

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            vulkan_ctx.device.clone(),
            Default::default(),
        ));

        let render_pass = Self::create_render_pass(vulkan_ctx.device.clone(), swapchain.clone())?;

        let framebuffers = Self::create_framebuffers(
            swapchain.clone(),
            &swapchain_image_views,
            render_pass.clone(),
        )?;

        let triangle_vertex_shader = triangle_vertex_shader::load(vulkan_ctx.device.clone())?;
        let triangle_fragment_shader = triangle_fragment_shader::load(vulkan_ctx.device.clone())?;

        let (triangle_pipeline, triangle_pipeline_layout) = Self::create_triangle_pipeline(
            vulkan_ctx.device.clone(),
            swapchain.clone(),
            render_pass.clone(),
            triangle_vertex_shader.clone(),
            triangle_fragment_shader.clone(),
        )?;

        let sync = Some(sync::now(vulkan_ctx.device.clone()).boxed());

        Ok(Self {
            vulkan: vulkan_ctx,
            window,
            frame_number: 0,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            recreate_swapchain: false,
            command_buffer_allocator,
            render_pass,
            framebuffers,
            triangle_vertex_shader,
            triangle_fragment_shader,
            triangle_pipeline,
            triangle_pipeline_layout,
            sync,
        })
    }

    /// Compute all needed data and present into surface.
    pub fn draw(&mut self) -> Result<(), GraphicsError> {
        // Skip draw when the window size is zero.
        if self.window.inner_size().width == 0 || self.window.inner_size().height == 0 {
            return Ok(());
        }

        self.sync
            .as_mut()
            .ok_or(GraphicsError::SynchronizationNotInitialized)?
            .cleanup_finished();

        if self.recreate_swapchain {
            self.recretate_swapchain()?;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(next_image) => next_image,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(err) => return Err(GraphicsError::from(err)),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        // make clear value flash
        let flash = (self.frame_number as f32 / 120.0).sin().abs();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(ClearValue::Float([0.0, 0.0, flash, 1.0]))],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                vulkano::command_buffer::SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .bind_pipeline_graphics(self.triangle_pipeline.clone())?
            .draw(3, 1, 0, 0)?
            .end_render_pass(SubpassEndInfo::default())?;

        let command_buffer = builder.build()?;

        let future = self
            .sync
            .take()
            .ok_or(GraphicsError::SynchronizationNotInitialized)?
            .join(acquire_future)
            .then_execute(self.vulkan.queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.vulkan.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.sync = Some(future.boxed());
            }

            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.sync = Some(sync::now(self.vulkan.device.clone()).boxed());
            }

            Err(err) => return Err(GraphicsError::from(err)),
        };

        self.frame_number += 1;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GraphicsError {
    /// Error that happens when creating a window.
    #[error("Could not create Window: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    /// Error that happens when loading Vulkan library.
    #[error("Could not load Vulkan library: {0}")]
    LibraryLoading(#[from] vulkano::LoadingError),

    #[error("Could not validate: {0}")]
    VulkanValidation(#[from] vulkano::Validated<vulkano::VulkanError>),

    #[error("Vulkan runtime error: {0}")]
    Vulkan(#[from] vulkano::VulkanError),

    #[error("Could not find any suitable physical device")]
    NoSuitablePhysicalDevice,

    #[error("Could not create needed devices queues.")]
    NoDeviceQueues,

    #[error("Synchronization mechanism wasn't initialized due to an unknown reason.")]
    SynchronizationNotInitialized,

    #[error("Validation error: {0}")]
    Validation(#[from] Box<vulkano::ValidationError>),

    #[error("Error executing command buffer: {0}")]
    CommandBufferExecution(#[from] vulkano::command_buffer::CommandBufferExecError),

    #[error("Could not find shader entry point: {0}")]
    WrongShaderEntryPoint(&'static str),
}

impl Debug for Graphics {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

mod triangle_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/triangle/shader.vert",
    }
}

mod triangle_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/triangle/shader.frag",
    }
}
