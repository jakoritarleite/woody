#![allow(dead_code)]
use std::ffi::CStr;
use std::sync::Arc;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::Entry;
use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;
use thiserror::Error;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::graphics2::vulkan::framebuffer::generate_framebuffers;

use self::framebuffer::Framebuffer;
use self::renderpass::RenderPass;
use self::swapchain::SwapchainContext;

mod framebuffer;
mod image;
mod renderpass;
mod swapchain;

/// Vulkan graphics context.
pub(crate) struct VulkanContext {
    /// Reference to winit Window.
    window: Arc<Window>,

    /// Vulkan Instance.
    instance: ash::Instance,

    #[cfg(debug_assertions)]
    /// Vulkan debug utils messenger.
    _debug_messenger: vk::DebugUtilsMessengerEXT,

    /// Vulkan swapchain screen Surface.
    surface: Surface,

    /// Vulkan logical device.
    device: ash::Device,

    /// Vulkan graphics queue.
    queue: vk::Queue,

    /// Swapchain context.
    swapchain: SwapchainContext,

    /// Vulkan main renderpass.
    renderpass: RenderPass,

    /// Vulkan swapchain framebuffers.
    framebuffers: Vec<Framebuffer>,
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

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let swapchain = SwapchainContext::new(
            &instance,
            physical_device,
            &device,
            surface,
            &surface_loader,
            window.inner_size().width,
            window.inner_size().height,
        )?;

        let renderpass = RenderPass::new(
            &device,
            &swapchain,
            [0, 0, window.inner_size().width, window.inner_size().height],
            [0.0, 0.0, 0.2, 1.0],
            1.0,
            0,
        )?;

        let framebuffers = generate_framebuffers(&device, &renderpass, &swapchain)?;

        Ok(Self {
            window,
            instance,
            #[cfg(debug_assertions)]
            _debug_messenger: debug_callback,
            surface: surface_loader,
            device,
            queue,
            swapchain,
            renderpass,
            framebuffers,
        })
    }
}

#[derive(Debug, Error)]
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