use std::sync::Arc;

use vulkano::device::Device;
use vulkano::image::sampler::ComponentMapping;
use vulkano::image::sampler::ComponentSwizzle;
use vulkano::image::view::ImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::image::view::ImageViewType;
use vulkano::image::Image;
use vulkano::image::ImageSubresourceRange;
use vulkano::image::ImageUsage;
use vulkano::swapchain::CompositeAlpha;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::sync::Sharing;
use winit::window::Window;

use super::context::Graphics;
use super::GraphicsError;

impl Graphics {
    /// Creates a swapchain.
    pub fn create_swapchain(
        window: Arc<Window>,
        device: Arc<Device>,
        surface: Arc<Surface>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), GraphicsError> {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())?;

        // Choosing the internal format that the images will have.
        let (image_format, image_color_space) = device
            .physical_device()
            .surface_formats(&surface, Default::default())?[0];
        // .find(|(format, color_space)| {
        //      *format == Format::B8G8R8A8_SRGB && *color_space == ColorSpace::SrgbNonLinear
        // })

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
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                image_sharing: Sharing::Exclusive,
                composite_alpha: CompositeAlpha::Opaque,
                present_mode,
                ..Default::default()
            },
        )?;

        Ok((swapchain, images))
    }

    pub fn create_swapchain_image_views(
        swapchain: Arc<Swapchain>,
        images: &[Arc<Image>],
    ) -> Result<Vec<Arc<ImageView>>, GraphicsError> {
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
                ImageView::new(
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

    pub fn recretate_swapchain(&mut self) -> Result<(), GraphicsError> {
        if self.window.inner_size().width == 0 || self.window.inner_size().height == 0 {
            return Ok(());
        }

        let (swapchain, images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: self.window.inner_size().into(),
            ..self.swapchain.create_info()
        })?;

        self.swapchain = swapchain;
        self.swapchain_images = images;

        self.swapchain_image_views =
            Self::create_swapchain_image_views(self.swapchain.clone(), &self.swapchain_images)?;

        self.framebuffers = Self::create_framebuffers(
            self.swapchain.clone(),
            &self.swapchain_image_views,
            self.render_pass.clone(),
        )?;

        Ok(())
    }
}
