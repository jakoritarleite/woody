#[allow(clippy::missing_safety_doc)]
use std::collections::HashSet;

use onlyerror::Error;
use vulkanalia::loader::LibloadingLoader;
use vulkanalia::loader::LIBRARY;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::PhysicalDevice;
use vulkanalia::vk::Queue;
use vulkanalia::vk::SurfaceKHR;
use vulkanalia::window as vk_window;
use vulkanalia::Device;
use vulkanalia::Entry;
use vulkanalia::Instance;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use self::swapchain::SwapchainSupport;

mod swapchain;

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

#[derive(Debug)]
// TODO remove this
#[allow(dead_code)]
pub struct Renderer {
    pub(crate) window: Window,

    pub(crate) entry: Entry,
    pub(crate) instance: Instance,

    pub(crate) surface_khr: SurfaceKHR,

    pub(crate) swapchain_support: SwapchainSupport,

    pub(crate) physical_device: PhysicalDevice,
    pub(crate) logical_device: Device,

    pub(crate) graphics_queue: Queue,
    pub(crate) present_queue: Queue,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, RendererError> {
        let window = WindowBuilder::new()
            .with_title("Woody Engine")
            .with_inner_size(LogicalSize::new(1024, 768))
            .build(event_loop)
            .map_err(|err| RendererError::Window(err.to_string()))?;

        let (
            entry,
            instance,
            surface_khr,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
        ) = unsafe {
            let loader = LibloadingLoader::new(LIBRARY)
                .map_err(|error| RendererError::LibraryLoading(error.to_string()))?;

            let entry =
                Entry::new(loader).map_err(|error| RendererError::Entry(error.to_string()))?;
            let instance = Self::create_instance(&window, &entry)?;

            let surface = vk_window::create_surface(&instance, &window, &window)?;

            let mut physical = None;
            let mut family_indices_present_surface_khr = None;
            let mut family_indices_graphics = None;
            for physical_device in instance.enumerate_physical_devices()? {
                let physical_properties = instance.get_physical_device_properties(physical_device);
                let queue_family_properties =
                    instance.get_physical_device_queue_family_properties(physical_device);

                let mut present_surface_khr = None;
                for (index, _) in queue_family_properties.iter().enumerate() {
                    if instance.get_physical_device_surface_support_khr(
                        physical_device,
                        index as u32,
                        surface,
                    )? {
                        present_surface_khr = Some(index as u32);
                        break;
                    }
                }

                let graphics = queue_family_properties
                    .iter()
                    .position(|position| position.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                    .map(|i| i as u32);

                if let (None, None) = (graphics, present_surface_khr) {
                    // TODO use logging library.
                    println!(
                        "Skipping physical device ( {} ): Missing required queue families.",
                        physical_properties.device_name
                    );
                    continue;
                }

                family_indices_present_surface_khr = present_surface_khr;
                family_indices_graphics = graphics;

                let extensions: Vec<_> = instance
                    .enumerate_device_extension_properties(physical_device, None)?
                    .iter()
                    .map(|extension| extension.extension_name)
                    .collect();

                if !DEVICE_EXTENSIONS
                    .iter()
                    .all(|extension| extensions.contains(extension))
                {
                    println!("Skipping physical device ( {} ): Missing required device extensions: {:?}.", physical_properties.device_name, DEVICE_EXTENSIONS);
                    continue;
                }

                let formats =
                    instance.get_physical_device_surface_formats_khr(physical_device, surface)?;
                let present_modes = instance
                    .get_physical_device_surface_present_modes_khr(physical_device, surface)?;

                if formats.is_empty() || present_modes.is_empty() {
                    println!(
                        "Skipping physical device ( {} ): Insufficient swapchain support.",
                        physical_properties.device_name
                    );
                    continue;
                }

                physical = Some(physical_device);
                println!(
                    "Selected physical_device ( {} ).",
                    physical_properties.device_name
                );
                break;
            }

            if physical.is_none() {
                return Err(RendererError::NoSuitablePhysicalDevice);
            }

            let physical = physical.unwrap();

            let unique_indices = HashSet::from([
                family_indices_graphics.unwrap(),
                family_indices_present_surface_khr.unwrap(),
            ]);

            let queue_infos: Vec<_> = unique_indices
                .iter()
                .map(|_| {
                    vk::DeviceQueueCreateInfo::builder()
                        .queue_family_index(family_indices_graphics.unwrap())
                        .queue_priorities(&[1.0])
                })
                .collect();

            let features = vk::PhysicalDeviceFeatures::builder();

            let extensions: Vec<_> = DEVICE_EXTENSIONS
                .iter()
                .map(|extension| extension.as_ptr())
                .collect();

            let info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_infos)
                .enabled_layer_names(&[])
                .enabled_extension_names(&extensions)
                .enabled_features(&features)
                .build();

            let logical = instance.create_device(physical, &info, None)?;

            let graphics_queue = logical.get_device_queue(family_indices_graphics.unwrap(), 0);
            let present_queue =
                logical.get_device_queue(family_indices_present_surface_khr.unwrap(), 0);

            (
                entry,
                instance,
                surface,
                physical,
                logical,
                graphics_queue,
                present_queue,
            )
        };

        Ok(Self {
            window,
            entry,
            instance,
            surface_khr,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
        })
    }

    pub unsafe fn create_instance(
        window: &Window,
        entry: &Entry,
    ) -> Result<Instance, RendererError> {
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Woody Engine\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"Woody\0")
            .engine_version(vk::make_version(1, 0, 0))
            .build();

        let extensions: Vec<_> = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|extension| extension.as_ptr())
            .collect();

        let info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&[])
            .enabled_extension_names(&extensions);

        let instance = entry.create_instance(&info, None)?;

        Ok(instance)
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Could not create window: {0}")]
    Window(String),

    #[error("Could not load Vulkan library: {0}")]
    LibraryLoading(String),

    #[error("Could not create Vulkan entry: {0}")]
    Entry(String),

    #[error("Vulkan error code")]
    Vulkan(#[from] vk::ErrorCode),

    #[error("Could not find any suitable physical device.")]
    NoSuitablePhysicalDevice,

    #[error("Could not create logical device")]
    LogicalDeviceCreation,
}
