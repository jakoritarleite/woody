use std::sync::Arc;
use std::time::Duration;

use glam::vec3;
use glam::Mat4;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use raw_window_handle::HasDisplayHandle;
use smallvec::smallvec;
use vulkano::buffer::BufferCreateInfo;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::IndexBuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::Device;
use vulkano::device::DeviceCreateInfo;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::Queue;
use vulkano::device::QueueCreateInfo;
use vulkano::device::QueueFlags;
use vulkano::instance::debug::DebugUtilsMessageSeverity;
use vulkano::instance::debug::DebugUtilsMessageType;
use vulkano::instance::debug::DebugUtilsMessenger;
use vulkano::instance::debug::DebugUtilsMessengerCallback;
use vulkano::instance::debug::DebugUtilsMessengerCreateInfo;
use vulkano::instance::Instance;
use vulkano::instance::InstanceCreateInfo;
use vulkano::instance::InstanceExtensions;
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::vertex_input::VertexBuffersCollection;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::DynamicState;
use vulkano::swapchain::acquire_next_image;
use vulkano::swapchain::Surface;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::SwapchainPresentInfo;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::sync::Sharing;
use vulkano::Validated;
use vulkano::VulkanError;
use vulkano::VulkanLibrary;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::graphics::vertex::Vertex;
use crate::graphics::vulkan::buffer::Buffer;
use crate::graphics::vulkan::buffer::BufferSub;
use crate::graphics::vulkan::framebuffer::generate_framebuffers;
use crate::graphics::vulkan::renderpass::RenderPass;

use self::command_buffer::CommandBuffer;
use self::framebuffer::Framebuffer;
use self::shaders::object::ObjectShader;
use self::swapchain::SwapchainContext;

use super::GraphicsError;

mod buffer;
mod command_buffer;
mod framebuffer;
mod image;
mod pipeline;
mod renderpass;
mod shaders;
mod swapchain;

/// Vulkan graphics context.
pub struct VulkanContext {
    /// Reference to winit Window.
    window: Arc<Window>,

    /// Vulkan Instance.
    pub(super) instance: Arc<Instance>,

    #[cfg(debug_assertions)]
    /// Vulkan debug utils messenger.
    _debug_messenger: DebugUtilsMessenger,

    /// Vulkan swapchain screen Surface.
    pub(super) surface: Arc<Surface>,

    /// Vulkan logical device.
    pub(super) device: Arc<Device>,

    /// Vulkan graphics queue.
    pub(super) queue: Arc<Queue>,

    /// Vulkan memory allocator.
    pub(super) memory_allocator: Arc<StandardMemoryAllocator>,

    /// Command buffer allocator.
    ///
    /// This abstracts the usage of [Command Pools](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandPool.html).
    pub(super) command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    /// Vulkan descriptor set allocator.
    pub(super) descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    /// Vulkan swapchain.
    swapchain: SwapchainContext,

    /// Vulkan main render_pass.
    render_pass: RenderPass,

    /// Vulkan Swapchain framebuffers.
    framebuffers: Vec<Framebuffer>,

    /// Graphics CommandBuffers
    graphics_command_buffers: Vec<CommandBuffer>,

    /// Determines if the swapchain must be recreated.
    ///
    /// This is used when the window size changes.
    pub(crate) recreate_swapchain: bool,

    /// Dynamic viewport used when we resize window.
    pub(super) viewport: Viewport,

    /// The current image we're drawing to.
    image_index: u32,

    // TODO: refactor this to use "raw" vulkan synchronization instead of this vulkano mechanism.
    //
    // We'll need to use the PrimaryCommandBufferAbstract trait to actually execute the command
    // with command_buffer.execute_after();
    //
    /// Vulkano Synchronization mechanism.
    sync: Option<Box<dyn GpuFuture>>,
    swapchain_future: Option<SwapchainAcquireFuture>,

    object_shader: ObjectShader,

    object_vertex_buffer: Buffer<Vertex>,
    object_index_buffer: Buffer<u32>,
}

impl VulkanContext {
    /// Creates a new [`VulkanContext`] instance.
    pub fn new(event_loop: &EventLoop<()>, window: Arc<Window>) -> Result<Self, GraphicsError> {
        let library = VulkanLibrary::new()?;
        let required_extensions = Surface::required_extensions(event_loop)?;

        let extensions = InstanceExtensions {
            #[cfg(debug_assertions)]
            ext_debug_utils: true,
            ..required_extensions
        };

        debug!("Vulkan required extensions: {:?}", extensions);

        let info = InstanceCreateInfo {
            enabled_extensions: extensions,
            #[cfg(debug_assertions)]
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
            ..InstanceCreateInfo::application_from_cargo_toml()
        };
        let instance = Instance::new(library, info)?;

        #[cfg(debug_assertions)]
        let _debug_messenger = unsafe {
            DebugUtilsMessenger::new(
                instance.clone(),
                DebugUtilsMessengerCreateInfo {
                    message_severity: DebugUtilsMessageSeverity::VERBOSE
                        | DebugUtilsMessageSeverity::INFO
                        | DebugUtilsMessageSeverity::WARNING
                        | DebugUtilsMessageSeverity::ERROR,

                    message_type: DebugUtilsMessageType::GENERAL
                        | DebugUtilsMessageType::VALIDATION
                        | DebugUtilsMessageType::PERFORMANCE,

                    ..DebugUtilsMessengerCreateInfo::user_callback(
                        DebugUtilsMessengerCallback::new(
                            |message_severity, message_type, callback_data| match message_severity {
                                DebugUtilsMessageSeverity::VERBOSE => {
                                    trace!("({:?}) {}", message_type, callback_data.message)
                                }
                                DebugUtilsMessageSeverity::INFO => {
                                    info!("({:?}) {}", message_type, callback_data.message)
                                }

                                DebugUtilsMessageSeverity::WARNING => {
                                    warn!("({:?}) {}", message_type, callback_data.message)
                                }

                                DebugUtilsMessageSeverity::ERROR => {
                                    error!("({:?}) {}", message_type, callback_data.message)
                                }
                                _ => error!(
                                    "UNKNOWN MESSAGE SEVERITY ({:?}) {}",
                                    message_type, callback_data.message
                                ),
                            },
                        ),
                    )
                },
            )?
        };

        let surface = Surface::from_window(instance.clone(), window.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_separate_depth_stencil_layouts: true,
            ..DeviceExtensions::empty()
        };

        let device_features = Features {
            separate_depth_stencil_layouts: true,
            sampler_anisotropy: true,
            sample_rate_shading: true,
            ..Default::default()
        };

        debug!("Device extensions {:?}", device_extensions);
        debug!("Device features {:?}", device_features);

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|device| device.supported_extensions().contains(&device_extensions))
            .filter(|device| device.supported_features().contains(&device_features))
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

        info!(
            "Selected physical device ( {} : {:?} )",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                enabled_features: device_features,
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
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let swapchain = SwapchainContext::new(
            memory_allocator.clone(),
            device.clone(),
            surface.clone(),
            window.inner_size().width,
            window.inner_size().height,
        )?;

        let render_pass = RenderPass::new(
            device.clone(),
            &swapchain,
            [0, 0, window.inner_size().width, window.inner_size().height],
            [0.0, 0.0, 0.2, 1.0],
            1.0,
            0,
        )?;

        let framebuffers = generate_framebuffers(&render_pass, &swapchain)?;

        // Create one command buffer for each swapchain image.
        let mut graphics_command_buffers = Vec::with_capacity(swapchain.images.len());
        for _ in 0..=swapchain.images.len() {
            let command_buffer =
                CommandBuffer::new(command_buffer_allocator.clone(), queue_family_index)?;

            graphics_command_buffers.push(command_buffer);
        }

        info!("Graphics command buffers created");

        //////// TEMPORARY BUFFER TEST

        let vertices = [
            Vertex::new(vec3(0.0, -0.5, 0.0)),
            Vertex::new(vec3(0.5, 0.5, 0.0)),
            Vertex::new(vec3(0.0, 0.5, 0.0)),
            Vertex::new(vec3(0.5, -0.5, 0.0)),
        ];
        let indices = [2, 1, 0, 1, 3, 0];

        let mut object_vertex_buffer = Buffer::<Vertex>::new(
            memory_allocator.clone(),
            BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of::<Vertex>() as u64 * 1024 * 1024,
        )?;

        let mut object_index_buffer = Buffer::<u32>::new(
            memory_allocator.clone(),
            BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_DEVICE,
            std::mem::size_of::<Vertex>() as u64 * 1024 * 1024,
        )?;

        let staging_vertex_buffer = Buffer::new_initialized(
            memory_allocator.clone(),
            BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            &vertices,
        )?;

        let staging_index_buffer = Buffer::new_initialized(
            memory_allocator.clone(),
            BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            &indices,
        )?;

        object_vertex_buffer.copy_from(
            command_buffer_allocator.clone(),
            queue.clone(),
            staging_vertex_buffer,
            0,
            0,
        )?;

        object_index_buffer.copy_from(
            command_buffer_allocator.clone(),
            queue.clone(),
            staging_index_buffer,
            0,
            0,
        )?;

        //////// TEMPORARY BUFFER TEST

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [swapchain.image_width(), swapchain.image_height()],
            depth_range: 0f32..=1f32,
        };

        let sync = Some(sync::now(device.clone()).boxed());

        let object_shader = ObjectShader::new(
            memory_allocator.clone(),
            descriptor_set_allocator.clone(),
            device.clone(),
            &render_pass,
            swapchain.images.len() as u32,
        )?;

        Ok(VulkanContext {
            window,
            instance,
            #[cfg(debug_assertions)]
            _debug_messenger,
            surface,
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            swapchain,
            render_pass,
            framebuffers,
            graphics_command_buffers,
            recreate_swapchain: false,
            viewport,
            image_index: 0,
            sync,
            swapchain_future: None,
            object_shader,
            object_vertex_buffer,
            object_index_buffer,
        })
    }

    pub(crate) fn update_global_state(
        &mut self,
        projection: Mat4,
        view: Mat4,
    ) -> Result<(), GraphicsError> {
        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];

        self.object_shader.bind(command_buffer)?;

        let global_uniform_object = self.object_shader.global_uniform_object_mut();

        global_uniform_object.projection = projection;
        global_uniform_object.view = view;

        // Writting to the uniform buffer must happen after waiting for the acquire future and
        // cleaning up, otherwise the buffer is still going to be marked as in use by the device.
        self.swapchain_future.as_ref().unwrap().wait(None)?;
        self.sync.as_mut().unwrap().cleanup_finished();

        match self
            .object_shader
            .update_global_state(self.image_index, command_buffer)
        {
            Ok(_) => {}
            Err(err) => {
                error!("Error updating global state: {}", err);

                return Err(err);
            }
        }

        Ok(())
    }

    pub(crate) fn update_object(&mut self, model: Mat4) -> Result<(), GraphicsError> {
        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];

        self.object_shader.update_state(model, command_buffer)?;

        self.object_shader.bind(command_buffer)?;

        command_buffer
            .handle_mut()?
            .bind_vertex_buffers(0, self.object_vertex_buffer.handle().clone())?;

        command_buffer
            .handle_mut()?
            .bind_index_buffer(self.object_index_buffer.handle().clone())?;

        command_buffer.handle_mut()?.draw_indexed(
            self.object_index_buffer.handle().len() as u32,
            1,
            0,
            0,
            0,
        )?;

        Ok(())
    }

    pub(crate) fn begin_frame(&mut self) -> Result<bool, GraphicsError> {
        if self.recreate_swapchain {
            debug!("Entering swapchain recreation");

            let width = self.window.inner_size().width;
            let height = self.window.inner_size().height;

            self.swapchain.recreate(width, height)?;

            debug!("Updating viewport extent to window inner size");
            self.viewport.extent = self.window.inner_size().into();

            debug!("Updating render_pass extent");
            self.render_pass.update_extent(width, height);

            debug!("Recreating framebuffers");
            self.framebuffers = generate_framebuffers(&self.render_pass, &self.swapchain)?;

            // TODO: maybe recalculate the device depth format?

            self.recreate_swapchain = false;

            debug!("Skipping frame due to swapchain recreation");
            return Ok(false);
        }

        let (image_index, suboptimal, acquire_future) = match acquire_next_image(
            self.swapchain.handle.clone(),
            Some(Duration::from_nanos(u64::MAX - 1)),
        ) {
            Ok(next) => next,
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                self.recreate_swapchain = true;
                return Ok(false);
            }
            Err(err) => return Err(GraphicsError::from(err)),
        };

        // Wait until we have the swapchain image
        acquire_future.wait(None)?;

        self.image_index = image_index;
        self.swapchain_future = Some(acquire_future);

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];
        command_buffer.reset();
        command_buffer.begin(CommandBufferUsage::MultipleSubmit)?;

        command_buffer
            .handle_mut()?
            .set_viewport(0, smallvec![self.viewport.clone()])?;

        self.render_pass.begin(
            command_buffer,
            &self.framebuffers[self.image_index as usize],
        )?;

        Ok(true)
    }

    pub(crate) fn end_frame(&mut self) -> Result<(), GraphicsError> {
        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];

        self.render_pass.end(command_buffer)?;

        let ended_command_buffer = command_buffer.end()?;

        self.sync
            .as_mut()
            .ok_or(GraphicsError::SynchronizationNotInitialized)?
            .cleanup_finished();

        let swapchain_future = self
            .swapchain_future
            .take()
            .ok_or(GraphicsError::SynchronizationNotInitialized)?;

        let future = self
            .sync
            .take()
            .ok_or(GraphicsError::SynchronizationNotInitialized)?
            .join(swapchain_future)
            .then_execute(self.queue.clone(), ended_command_buffer)?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.handle.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        command_buffer.update_submitted();

        match future.map_err(Validated::unwrap) {
            Ok(future) => self.sync = Some(future.boxed()),

            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.sync = Some(sync::now(self.device.clone()).boxed());
            }

            Err(err) => return Err(GraphicsError::from(err)),
        }

        Ok(())
    }
}
