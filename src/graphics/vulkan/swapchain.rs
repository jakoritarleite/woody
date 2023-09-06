use std::sync::Arc;

use log::debug;
use log::info;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::format::FormatFeatures;
use vulkano::image::sampler::ComponentMapping;
use vulkano::image::sampler::ComponentSwizzle;
use vulkano::image::view::ImageView as vkImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::image::view::ImageViewType;
use vulkano::image::Image as vkImage;
use vulkano::image::ImageAspects;
use vulkano::image::ImageSubresourceRange;
use vulkano::image::ImageTiling;
use vulkano::image::ImageType;
use vulkano::image::ImageUsage;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::CompositeAlpha;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::sync::Sharing;

use crate::graphics::vulkan::image::Image;
use crate::graphics::GraphicsError;

const CANDIDATE_FORMATS: [Format; 3] = [
    Format::D32_SFLOAT,
    Format::D32_SFLOAT_S8_UINT,
    Format::D24_UNORM_S8_UINT,
];

/// Abstraction of the Vulkan Swapchain with usefull methods.
pub(super) struct SwapchainContext {
    handle: Arc<Swapchain>,
    images: Vec<Arc<vkImage>>,
    image_views: Vec<Arc<vkImageView>>,
    depth_format: Format,
    depth_attachment: Image,
}

// TODO: check for duplicated code with the [image](crate::graphics::vulkan::image) module.
impl SwapchainContext {
    /// Creates a new instance of [`SwapchainContext`].
    pub(super) fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        device: Arc<Device>,
        surface: Arc<Surface>,
        width: u32,
        height: u32,
    ) -> Result<Self, GraphicsError> {
        let (swapchain, images) = Self::create_swapchain(device.clone(), surface, width, height)?;
        let image_views = Self::create_swapchain_image_views(swapchain.clone(), &images)?;

        info!("Created swapchain with extent: ({}, {})", width, height);

        debug!("Searching for supported depth format");

        let mut depth_format = None;

        for format in CANDIDATE_FORMATS.into_iter() {
            debug!("Checking if device supports ({:?}) format", format);

            let format_properties = device.physical_device().format_properties(format)?;

            if format_properties
                .linear_tiling_features
                .intersects(FormatFeatures::DEPTH_STENCIL_ATTACHMENT)
            {
                depth_format = Some(format);
            } else if format_properties
                .optimal_tiling_features
                .intersects(FormatFeatures::DEPTH_STENCIL_ATTACHMENT)
            {
                depth_format = Some(format);
            }
        }

        let depth_format = match depth_format {
            Some(format) => format,
            None => return Err(GraphicsError::NoSupportedDepthFormat),
        };

        info!("Found supported depth format: ({:?})", depth_format);

        debug!("Creating depth attachment");

        let depth_attachment = Image::new(
            memory_allocator,
            ImageType::Dim2d,
            depth_format,
            ImageTiling::Optimal,
            ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            ImageAspects::DEPTH,
            width,
            height,
        )?;

        Ok(Self {
            handle: swapchain,
            images,
            image_views,
            depth_format,
            depth_attachment,
        })
    }

    /// Returns the image extent width.
    pub(super) fn image_width(&self) -> f32 {
        self.handle.image_extent()[0] as f32
    }

    /// Returns the image extent height.
    pub(super) fn image_height(&self) -> f32 {
        self.handle.image_extent()[1] as f32
    }

    /// Creates a new [Vulkan Swapchain](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSwapchainKHR.html) and it's Images.
    fn create_swapchain(
        device: Arc<Device>,
        surface: Arc<Surface>,
        width: u32,
        height: u32,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<vkImage>>), GraphicsError> {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())?;

        // Choosing the internal format that the images will have.
        let (image_format, image_color_space) = device
            .physical_device()
            .surface_formats(&surface, Default::default())?[0];

        // Check if Surface supports using Mailbox, if not use Fifo.
        let present_mode = device
            .physical_device()
            .surface_present_modes(&surface)?
            .find(|mode| *mode == PresentMode::Mailbox)
            .unwrap_or(PresentMode::Fifo);

        let (swapchain, images) = Swapchain::new(
            device,
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_color_space,
                image_extent: [width, height],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                image_sharing: Sharing::Exclusive,
                composite_alpha: CompositeAlpha::Opaque,
                present_mode,
                ..Default::default()
            },
        )?;

        Ok((swapchain, images))
    }

    /// Recreates the swapchain for the new window width and height target.
    ///
    /// Reuses the same configuration from the current swapchain in `handle` field.
    pub(super) fn recreate_swapchain(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), GraphicsError> {
        let (swapchain, images) = self.handle.recreate(SwapchainCreateInfo {
            image_extent: [width, height],
            ..self.handle.create_info()
        })?;

        let image_views = Self::create_swapchain_image_views(swapchain.clone(), &images)?;

        self.handle = swapchain;
        self.images = images;
        self.image_views = image_views;

        Ok(())
    }

    /// Creates [Vulkan ImageView](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageView.html) from current swapchain images.
    fn create_swapchain_image_views(
        swapchain: Arc<Swapchain>,
        images: &[Arc<vkImage>],
    ) -> Result<Vec<Arc<vkImageView>>, GraphicsError> {
        let components = ComponentMapping {
            r: ComponentSwizzle::Identity,
            g: ComponentSwizzle::Identity,
            b: ComponentSwizzle::Identity,
            a: ComponentSwizzle::Identity,
        };
        let subresource_range =
            ImageSubresourceRange::from_parameters(swapchain.image_format(), 1, 1);

        let image_views = images
            .iter()
            .map(|image| {
                vkImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Dim2d,
                        format: swapchain.image_format(),
                        component_mapping: components,
                        subresource_range: subresource_range.clone(),
                        ..Default::default()
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(image_views)
    }
}
