use ash::extensions::khr::Surface;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::Device;
use ash::Instance;

use super::Error;

const CANDIDATE_FORMATS: [vk::Format; 3] = [
    vk::Format::D32_SFLOAT,
    vk::Format::D32_SFLOAT_S8_UINT,
    vk::Format::D24_UNORM_S8_UINT,
];

/// Abstraction of the Vulkan Swapchain.
pub struct SwapchainContext {
    khr: vk::SwapchainKHR,
    handle: Swapchain,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    depth_format: vk::Format,
    // TODO: create depth attachment
    //depth_attachment: Image
}

impl SwapchainContext {
    /// Creates a new instance of [`SwapchainContext`].
    pub fn new(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        device: &Device,
        surface_khr: vk::SurfaceKHR,
        surface: &Surface,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let surface_capabilities = unsafe {
            surface.get_physical_device_surface_capabilities(*physical_device, surface_khr)?
        };

        // Chosing the internal format that the images will have.
        let vk::SurfaceFormatKHR {
            format,
            color_space,
        } = unsafe {
            surface.get_physical_device_surface_formats(*physical_device, surface_khr)?[0]
        };

        // Check if Surface supports using Mailbox, if not use Fifo.
        let present_mode = unsafe {
            surface
                .get_physical_device_surface_present_modes(*physical_device, surface_khr)?
                .into_iter()
                .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .min_image_count(surface_capabilities.min_image_count.max(2))
            .image_format(format)
            .image_color_space(color_space)
            .image_extent(vk::Extent2D { width, height })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode);

        let loader = Swapchain::new(instance, device);
        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { loader.get_swapchain_images(swapchain)? };
        let image_views = images
            .iter()
            .map(|image| {
                let subresource_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR
                        | vk::ImageAspectFlags::DEPTH
                        | vk::ImageAspectFlags::STENCIL,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };

                let create_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(subresource_range)
                    .image(*image);

                unsafe { device.create_image_view(&create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        log::info!("Created swapchain with extent: ({}, {})", width, height);

        let mut depth_format = None;

        for format in CANDIDATE_FORMATS.into_iter() {
            let format_properties =
                unsafe { instance.get_physical_device_format_properties(*physical_device, format) };

            let linear_tiling_contains_depth_stencil_attachment = format_properties
                .linear_tiling_features
                .intersects(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT);

            let optimal_tiling_contains_depth_stencil_attachment = format_properties
                .optimal_tiling_features
                .intersects(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT);

            if linear_tiling_contains_depth_stencil_attachment
                || optimal_tiling_contains_depth_stencil_attachment
            {
                depth_format = Some(format);
            }
        }

        let depth_format = match depth_format {
            Some(format) => format,
            None => return Err(Error::NoSupportedDepthFormat),
        };

        log::info!("Found supported depth format ({:?})", depth_format);

        Ok(Self {
            khr: swapchain,
            handle: loader,
            images,
            image_views,
            image_format: format,
            image_color_space: color_space,
            depth_format,
        })
    }
}
