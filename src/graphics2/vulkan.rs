#![allow(dead_code)]
use std::ffi::CStr;
use std::sync::Arc;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::Entry;
use itertools::Itertools;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;
use thiserror::Error;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::graphics2::vulkan::command_buffer::CommandBufferLevel;
use crate::graphics2::vulkan::command_buffer::CommandPoolCreateFlags;
use crate::graphics2::vulkan::framebuffer::generate_framebuffers;
use crate::graphics2::vulkan::sync::FenceCreateFlags;
use crate::graphics2::RenderArea;
use crate::graphics2::Rgba;

use self::command_buffer::CommandBuffer;
use self::command_buffer::CommandBufferUsage;
use self::command_buffer::CommandPool;
use self::framebuffer::Framebuffer;
use self::renderpass::RenderPass;
use self::swapchain::SwapchainContext;
use self::sync::Fence;

mod command_buffer;
mod framebuffer;
mod image;
mod renderpass;
mod swapchain;
mod sync;

/// Vulkan graphics context.
pub(crate) struct VulkanContext {
    /// Reference to winit Window.
    window: Arc<Window>,

    /// Vulkan Instance.
    instance: Arc<ash::Instance>,

    #[cfg(debug_assertions)]
    /// Vulkan debug utils messenger.
    _debug_loader: DebugUtils,
    #[cfg(debug_assertions)]
    _debug_messenger: vk::DebugUtilsMessengerEXT,

    /// Vulkan swapchain screen Surface.
    surface: Surface,
    surface_khr: vk::SurfaceKHR,

    /// Vulkan logical device.
    device: Arc<ash::Device>,

    /// Vulkan graphics queue.
    queue: vk::Queue,

    /// Swapchain context.
    swapchain: SwapchainContext,

    /// Determines if the swapchain must be recreated.
    ///
    /// This is used when the window size changes.
    pub(super) recreate_swapchain: bool,

    /// Vulkan main renderpass.
    renderpass: RenderPass,

    /// Vulkan swapchain framebuffers.
    framebuffers: Vec<Framebuffer>,

    /// Vulkan command pool.
    command_pool: CommandPool,

    /// Vulkan graphics command buffers.
    /// Note: there's one command buffer for each swapchain image.
    graphics_command_buffers: Vec<CommandBuffer>,

    /// Represents when an image is available to be rendered to.
    image_available_semaphores: Vec<vk::Semaphore>,

    /// Represents when a queue is ready to be presented.
    queue_complete_semaphores: Vec<vk::Semaphore>,

    in_flight_fence_count: u32,
    in_flight_fences: Vec<Arc<Fence>>,

    /// This actually holds pointers to the [`Fence`]s owned by [`Self::in_flight_fences`].
    images_in_flight: Vec<Option<Arc<Fence>>>,

    current_frame: u8,

    /// The current image we're drawing to.
    image_index: u32,

    /// Dynamic viewport used when we resize window.
    viewport: vk::Viewport,
}

impl VulkanContext {
    /// Creates a new [`VulkanContext`] instance.
    pub fn new(_event_loop: &EventLoop<()>, window: Arc<Window>) -> Result<Self, Error> {
        let entry = Entry::linked();
        let application_info = vk::ApplicationInfo::builder()
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .application_name(unsafe { CStr::from_ptr(b"Woody Engine\0".as_ptr().cast()) });

        let mut extensions =
            ash_window::enumerate_required_extensions(window.raw_display_handle())?.to_vec();
        extensions.push(DebugUtils::name().as_ptr());

        log::debug!(
            "Vulkan loaded extensions: {:?}",
            debug_str_raw_pointers(&extensions)
        );

        let layers = [
            #[cfg(debug_assertions)]
            "VK_LAYER_KHRONOS_validation\0".as_ptr().cast(),
        ];

        log::debug!(
            "Vulkan loaded layers: {:?}",
            debug_str_raw_pointers(&layers)
        );

        let instance = unsafe {
            entry.create_instance(
                &vk::InstanceCreateInfo::builder()
                    .application_info(&application_info)
                    .enabled_extension_names(&extensions)
                    .enabled_layer_names(&layers),
                None,
            )?
        };
        let instance = Arc::new(instance);

        #[cfg(debug_assertions)]
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            )
            .pfn_user_callback(Some(vk_debug_callback));

        #[cfg(debug_assertions)]
        let debug_utils_loader = DebugUtils::new(&entry, &instance);

        #[cfg(debug_assertions)]
        let debug_callback =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let surface_loader = Surface::new(&entry, &instance);

        let (physical_device, queue_family_index) = unsafe {
            instance
                .enumerate_physical_devices()?
                .iter()
                .flat_map(|device| {
                    instance
                        .get_physical_device_queue_family_properties(*device)
                        .iter()
                        .map(|props| (*device, *props))
                        .collect::<Vec<_>>()
                })
                .enumerate()
                .filter(|(_, (_, props))| props.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .filter_map(|(index, (device, _))| {
                    if surface_loader
                        .get_physical_device_surface_support(device, index as u32, surface)
                        .is_ok()
                    {
                        return Some((device, index as u32));
                    }

                    None
                })
                .min_by_key(|(device, _)| {
                    match instance.get_physical_device_properties(*device).device_type {
                        vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                        vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                        vk::PhysicalDeviceType::CPU => 3,
                        vk::PhysicalDeviceType::OTHER => 4,
                        _ => 5,
                    }
                })
                .ok_or(Error::NoSuitablePhysicalDevice)?
        };

        let device_props = unsafe { instance.get_physical_device_properties(physical_device) };

        log::info!(
            "Selected physical device ( {:?} : {:?} )",
            CStr::from_bytes_until_nul(
                &device_props
                    .device_name
                    .iter()
                    .map(|char| *char as u8)
                    .collect::<Vec<_>>(),
            )
            .expect("Invalid device name"),
            device_props.device_type
        );

        let device_extensions = [
            Swapchain::name().as_ptr(),
            "VK_KHR_separate_depth_stencil_layouts\0".as_ptr().cast(),
        ];
        let device_features = vk::PhysicalDeviceFeatures::default();

        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0]);

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_extension_names(&device_extensions)
            .enabled_features(&device_features);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
        let device = Arc::new(device);

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let swapchain = SwapchainContext::new(
            instance.clone(),
            physical_device,
            device.clone(),
            surface,
            &surface_loader,
            queue_family_index,
            window.inner_size().width,
            window.inner_size().height,
        )?;

        let renderpass = RenderPass::new(
            device.clone(),
            &swapchain,
            RenderArea::from(window.inner_size()),
            Rgba(0.0, 0.0, 0.2, 1.0),
            1.0,
            0,
        )?;

        let framebuffers = generate_framebuffers(&device, &renderpass, &swapchain)?;

        let command_pool = CommandPool::new(
            device.clone(),
            queue_family_index,
            CommandPoolCreateFlags::ResetCommandBuffer,
        )?;

        // Create one commend buffer for each swapchain image.
        let graphics_command_buffers = (0..=SwapchainContext::MAX_FRAMES_IN_FLIGHT)
            .map(|_| command_pool.allocate(CommandBufferLevel::Primary))
            .collect::<Result<Vec<_>, _>>()?;

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

        let (image_available_semaphores, queue_complete_semaphores, in_flight_fences) = (0
            ..=SwapchainContext::MAX_FRAMES_IN_FLIGHT)
            .map(|_| unsafe {
                (
                    device.create_semaphore(&semaphore_create_info, None),
                    device.create_semaphore(&semaphore_create_info, None),
                    Fence::new(device.clone(), FenceCreateFlags::Signaled),
                )
            })
            .multiunzip::<(Vec<_>, Vec<_>, Vec<_>)>();

        let image_available_semaphores = image_available_semaphores
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let queue_complete_semaphores = queue_complete_semaphores
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let in_flight_fences = in_flight_fences
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();

        let images_in_flight = vec![None; swapchain.images.len()];

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: window.inner_size().width as f32,
            height: window.inner_size().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        Ok(Self {
            window,
            instance,
            #[cfg(debug_assertions)]
            _debug_loader: debug_utils_loader,
            #[cfg(debug_assertions)]
            _debug_messenger: debug_callback,
            surface: surface_loader,
            surface_khr: surface,
            device,
            queue,
            swapchain,
            recreate_swapchain: false,
            renderpass,
            framebuffers,
            command_pool,
            graphics_command_buffers,
            image_available_semaphores,
            queue_complete_semaphores,
            in_flight_fence_count: in_flight_fences.len() as _,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
            image_index: 0,
            viewport,
        })
    }

    pub fn begin_frame(&mut self) -> Result<(), Error> {
        if self.recreate_swapchain {
            unsafe {
                self.device.device_wait_idle()?;
            }

            self.images_in_flight = vec![None; self.swapchain.images.len()];

            let width = self.window.inner_size().width;
            let height = self.window.inner_size().height;

            log::trace!("Recreating swapchain");
            self.swapchain.recreate_swapchain(width, height)?;

            self.viewport.width = width as f32;
            self.viewport.height = height as f32;

            log::trace!("Updating Renderpass RenderArea");
            *self.renderpass.render_area_mut() = self.window.inner_size().into();

            log::trace!("Recreating framebuffers");
            for framebuffer in self.framebuffers.iter() {
                unsafe { self.device.destroy_framebuffer(framebuffer.handle, None) };
            }

            self.framebuffers =
                generate_framebuffers(&self.device, &self.renderpass, &self.swapchain)?;

            self.recreate_swapchain = false;

            return Err(Error::Unpresentable);
        }

        // Wait for the current frame to complete.
        if !self.in_flight_fences[self.current_frame as usize].wait(u64::MAX)? {
            log::warn!("In-flight fence wait failed.");

            return Err(Error::Unpresentable);
        }

        self.image_index = match self.swapchain.acquire_next_image(
            u64::MAX,
            self.image_available_semaphores[self.current_frame as usize],
            None,
        ) {
            Ok((index, suboptimal)) => {
                if suboptimal {
                    self.recreate_swapchain = true;
                }
                index
            }

            Err(Error::OutOfDate) | Err(Error::Suboptimal) => {
                self.recreate_swapchain = true;
                return Err(Error::Unpresentable);
            }

            Err(error) => return Err(error),
        };

        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];
        command_buffer.begin(CommandBufferUsage::MultipleSubmit)?;

        unsafe {
            self.device
                .cmd_set_viewport(command_buffer.handle, 0, &[self.viewport]);
            self.device.cmd_set_scissor(
                command_buffer.handle,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D {
                        width: self.window.inner_size().width,
                        height: self.window.inner_size().height,
                    },
                }],
            )
        }

        self.renderpass.begin(
            command_buffer,
            &self.framebuffers[self.image_index as usize],
        );

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), Error> {
        let command_buffer = &mut self.graphics_command_buffers[self.image_index as usize];

        self.renderpass.end(command_buffer);
        command_buffer.end()?;

        if let Some(images_in_flight) = &self.images_in_flight[self.image_index as usize] {
            // Make sure the previous frame is not using this image.
            images_in_flight.wait(u64::MAX)?;
        }

        // Mark the image fence as in-use by this frame.
        self.images_in_flight[self.image_index as usize] =
            Some(self.in_flight_fences[self.current_frame as usize].clone());

        // Reset the fence for use in the next frame.
        self.in_flight_fences[self.current_frame as usize].reset()?;

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&command_buffer.handle))
            .signal_semaphores(std::slice::from_ref(
                &self.queue_complete_semaphores[self.current_frame as usize],
            ))
            .wait_semaphores(std::slice::from_ref(
                &self.image_available_semaphores[self.current_frame as usize],
            ))
            .wait_dst_stage_mask(std::slice::from_ref(
                &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ));

        unsafe {
            self.device.queue_submit(
                self.queue,
                std::slice::from_ref(&submit_info),
                self.in_flight_fences[self.current_frame as usize].handle,
            )?;

            match self.swapchain.present(
                self.queue,
                self.queue_complete_semaphores[self.current_frame as usize],
                self.image_index,
            ) {
                Ok(_) | Err(Error::OutOfDate) | Err(Error::Suboptimal) => {}
                err @ Err(_) => err?,
            };
        }

        self.current_frame += 1;
        self.current_frame %= SwapchainContext::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum Error {
    #[error(r#"Vulkan returned an error: {0}.
See https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkResult.html for more information."#)]
    VkResult(#[from] vk::Result),

    #[error("Could not find any suitable physical device")]
    NoSuitablePhysicalDevice,

    #[error("Device does not support any candidate depth formats")]
    NoSupportedDepthFormat,

    #[error("Could not find a suitable memory index")]
    NoSuitableMemoryIndex,

    #[error("Swapchain no longer matches Surface but can still be used.")]
    Suboptimal,

    #[error("Swapchain is out of date with Surface and must be recreated.")]
    OutOfDate,

    #[error("The current frame is not presentable, and so it must be skipped.")]
    Unpresentable,
}

unsafe extern "system" fn vk_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*callback_data).p_message).to_string_lossy();

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::trace!("({:?}) {}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("({:?}) {}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("({:?}) {}", message_type, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("({:?}) {}", message_type, message)
        }
        _ => log::error!("UNKNOWN MESSAGE SEVERITY ({:?}) {}", message_type, message),
    }

    vk::FALSE
}

#[inline]
fn debug_str_raw_pointers(ptrs: &[*const i8]) -> Vec<&CStr> {
    ptrs.iter()
        .map(|ptr| unsafe { CStr::from_ptr(*ptr) })
        .collect()
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
        }

        for fence in self.in_flight_fences.iter() {
            unsafe { self.device.destroy_fence(fence.handle, None) };
        }

        for semaphore in self.queue_complete_semaphores.iter() {
            unsafe { self.device.destroy_semaphore(*semaphore, None) };
        }

        for semaphore in self.image_available_semaphores.iter() {
            unsafe { self.device.destroy_semaphore(*semaphore, None) };
        }

        unsafe {
            self.device
                .destroy_command_pool(self.command_pool.handle, None)
        };

        for framebuffer in self.framebuffers.iter() {
            unsafe {
                self.device.destroy_framebuffer(framebuffer.handle, None);
            };
        }

        unsafe {
            self.device
                .destroy_render_pass(self.renderpass.handle, None);

            self.device
                .free_memory(self.swapchain.depth_attachment.memory, None);

            self.device
                .destroy_image_view(self.swapchain.depth_attachment.view, None);

            self.device
                .destroy_image(self.swapchain.depth_attachment.image, None);
        }

        for swapchain_image_view in self.swapchain.image_views.iter() {
            unsafe {
                self.device.destroy_image_view(*swapchain_image_view, None);
            }
        }

        unsafe {
            self.swapchain
                .handle
                .destroy_swapchain(self.swapchain.khr, None);

            self.surface.destroy_surface(self.surface_khr, None);

            #[cfg(debug_assertions)]
            self._debug_loader
                .destroy_debug_utils_messenger(self._debug_messenger, None);

            self.device.destroy_device(None);
        }
    }
}
