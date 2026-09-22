use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use super::device::Device;
use super::device::PhysicalDevice;
use super::image::Image;
use super::image::ImageCreateInfo;
use super::instance::Instance;
use super::sync::Fence;
use super::Error;

const CANDIDATE_FORMATS: [vk::Format; 3] = [
    vk::Format::D32_SFLOAT,
    vk::Format::D32_SFLOAT_S8_UINT,
    vk::Format::D24_UNORM_S8_UINT,
];

/// Abstraction of the Vulkan Swapchain.
pub struct SwapchainContext {
    pub khr: vk::SwapchainKHR,
    pub handle: khr::Swapchain,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    create_info: vk::SwapchainCreateInfoKHR,
    pub depth_format: vk::Format,
    pub depth_attachment: Image,
    pub extent: vk::Extent2D,
    _instance: Arc<Instance>,
    _device: Arc<Device>,
    _surface: Arc<khr::Surface>,
    _surface_khr: vk::SurfaceKHR,
}

impl SwapchainContext {
    pub const MAX_FRAMES_IN_FLIGHT: u8 = 2;

    /// Creates a new instance of [`SwapchainContext`].
    pub fn new(
        instance: Arc<Instance>,
        device: Arc<Device>,
        surface_khr: vk::SurfaceKHR,
        surface: Arc<khr::Surface>,
        queue_family_index: u32,
        extent: vk::Extent2D,
    ) -> Result<Self, Error> {
        let surface_capabilities = unsafe {
            surface.get_physical_device_surface_capabilities(
                device.physical_device().handle,
                surface_khr,
            )?
        };

        // Chosing the internal format that the images will have.
        let vk::SurfaceFormatKHR {
            format,
            color_space,
        } = unsafe {
            surface
                .get_physical_device_surface_formats(device.physical_device().handle, surface_khr)?
                [0]
        };

        // Check if Surface supports using Mailbox, if not use Fifo.
        let present_mode = unsafe {
            surface
                .get_physical_device_surface_present_modes(
                    device.physical_device().handle,
                    surface_khr,
                )?
                .into_iter()
                .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };

        let clampped_extent = vk::Extent2D {
            width: clamp(
                extent.width,
                surface_capabilities.min_image_extent.width,
                surface_capabilities.max_image_extent.width,
            ),
            height: clamp(
                extent.height,
                surface_capabilities.min_image_extent.height,
                surface_capabilities.max_image_extent.height,
            ),
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .min_image_count(surface_capabilities.min_image_count.max(2))
            .image_format(format)
            .image_color_space(color_space)
            .image_extent(clampped_extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .surface(surface_khr)
            .pre_transform(surface_capabilities.current_transform)
            .image_array_layers(surface_capabilities.max_image_array_layers)
            .queue_family_indices(std::slice::from_ref(&queue_family_index))
            .clipped(true);

        let loader = khr::Swapchain::new(&instance, &device);
        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { loader.get_swapchain_images(swapchain)? };
        let image_views = images
            .iter()
            .map(|image| {
                let subresource_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
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

        log::info!(
            "Created swapchain with extent: ({}, {})",
            extent.width,
            extent.height
        );

        let mut depth_format = None;

        for format in CANDIDATE_FORMATS.into_iter() {
            let format_properties = unsafe {
                instance
                    .get_physical_device_format_properties(device.physical_device().handle, format)
            };

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

        let depth_attachment = Image::new(
            &instance,
            &device,
            ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: depth_format,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                aspect_mask: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
                extent: vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
            },
        )?;

        Ok(Self {
            khr: swapchain,
            handle: loader,
            images,
            image_views,
            create_info: *swapchain_create_info,
            depth_format,
            depth_attachment,
            extent,
            _instance: instance,
            _device: device,
            _surface: surface,
            _surface_khr: surface_khr,
        })
    }

    /// Recreates the swapchain.
    pub fn recreate_swapchain(&mut self, extent: vk::Extent2D) -> Result<(), Error> {
        unsafe {
            self._device.device_wait_idle()?;

            self._device.free_memory(self.depth_attachment.memory, None);
            self._device
                .destroy_image_view(self.depth_attachment.view, None);
            self._device
                .destroy_image(self.depth_attachment.image, None);

            for image_view in self.image_views.iter() {
                self._device.destroy_image_view(*image_view, None);
            }

            self.handle.destroy_swapchain(self.khr, None);
        }

        if extent.width == 0 || extent.height == 0 {
            log::info!("Ignoring swapchain recreation due to one of dimensions being 0");
            return Ok(());
        }

        let surface_capabilities = unsafe {
            self._surface.get_physical_device_surface_capabilities(
                self._device.physical_device().handle,
                self._surface_khr,
            )?
        };

        let clampped_extent = vk::Extent2D {
            width: clamp(
                extent.width,
                surface_capabilities.min_image_extent.width,
                surface_capabilities.max_image_extent.width,
            ),
            height: clamp(
                extent.height,
                surface_capabilities.min_image_extent.height,
                surface_capabilities.max_image_extent.height,
            ),
        };

        // TODO: check if we need to query the capabilities again.
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            image_extent: clampped_extent,
            ..self.create_info
        };

        let swapchain = unsafe { self.handle.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { self.handle.get_swapchain_images(swapchain)? };
        let image_views = images
            .iter()
            .map(|image| {
                let subresource_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };

                let create_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.image_format())
                    .subresource_range(subresource_range)
                    .image(*image);

                unsafe { self._device.create_image_view(&create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let depth_attachment = Image::new(
            &self._instance,
            &self._device,
            ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: self.depth_format,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                aspect_mask: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
                extent: vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
            },
        )?;

        self.create_info = swapchain_create_info;
        self.khr = swapchain;
        self.images = images;
        self.image_views = image_views;
        self.depth_attachment = depth_attachment;
        self.extent = extent;

        Ok(())
    }

    /// Acquire next image to be used.
    pub fn acquire_next_image(
        &mut self,
        timeout: u64,
        semaphore: vk::Semaphore,
        fence: Option<Fence>,
    ) -> Result<(u32, bool), Error> {
        let next_image = unsafe {
            self.handle.acquire_next_image(
                self.khr,
                timeout,
                semaphore,
                fence
                    .map(|fence| fence.handle)
                    .unwrap_or_else(vk::Fence::null),
            )
        };

        match next_image {
            Ok(next) => Ok(next),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(Error::OutOfDate),
            Err(vk::Result::SUBOPTIMAL_KHR) => Err(Error::Suboptimal),
            Err(error) => Err(Error::from(error)),
        }
    }

    pub fn present(
        &mut self,
        graphics_queue: vk::Queue,
        semaphore: vk::Semaphore,
        image_index: u32,
    ) -> Result<(), Error> {
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(std::slice::from_ref(&self.khr))
            .image_indices(std::slice::from_ref(&image_index))
            .wait_semaphores(std::slice::from_ref(&semaphore));

        let result = unsafe { self.handle.queue_present(graphics_queue, &present_info) };

        match result {
            Ok(_) => Ok(()),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err(Error::OutOfDate),
            Err(vk::Result::SUBOPTIMAL_KHR) => Err(Error::Suboptimal),
            Err(error) => Err(Error::from(error)),
        }
    }

    /// Returns the swapchain image format.
    pub fn image_format(&self) -> vk::Format {
        self.create_info.image_format
    }
}

fn clamp<T: Ord>(input: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
