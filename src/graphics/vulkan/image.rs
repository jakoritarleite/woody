use std::sync::Arc;

use vulkano::format::Format;
use vulkano::image::view::ImageView as vkImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::image::view::ImageViewType;
use vulkano::image::Image as vkImage;
use vulkano::image::ImageAspects;
use vulkano::image::ImageCreateInfo;
use vulkano::image::ImageLayout;
use vulkano::image::ImageSubresourceRange;
use vulkano::image::ImageTiling;
use vulkano::image::ImageType;
use vulkano::image::ImageUsage;
use vulkano::image::SampleCount;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::Sharing;

use crate::graphics::GraphicsError;

/// Abstraction of the Vulkan Image and Image view.
pub(super) struct Image {
    handle: Arc<vkImage>,
    view: Arc<vkImageView>,
    width: u32,
    height: u32,
}

impl Image {
    /// Creates a new instance of [`Image`].
    pub(super) fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        image_type: ImageType,
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsage,
        view_aspects: ImageAspects,
        width: u32,
        height: u32,
    ) -> Result<Self, GraphicsError> {
        let image = Self::create_image(
            memory_allocator,
            image_type,
            format,
            tiling,
            usage,
            width,
            height,
        )?;
        // TODO: make creating the view configurable ??
        let view = Self::create_image_view(image.clone(), format, view_aspects)?;

        Ok(Self {
            handle: image,
            view,
            width,
            height,
        })
    }

    /// Creates a configurable [vkImage](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImage.html).
    pub(super) fn create_image(
        memory_allocator: Arc<StandardMemoryAllocator>,
        image_type: ImageType,
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsage,
        width: u32,
        height: u32,
    ) -> Result<Arc<vkImage>, GraphicsError> {
        // TODO:
        // - extent:        support configurable depth.
        // - array_layers:  support configurable number of layers in the image.
        // - mip_levels:    support configurable mip mapping.
        // - samples:       support configurable sample count.
        // - sharing:       support configurable sharing mode.
        let info = ImageCreateInfo {
            image_type,
            format,
            extent: [width, height, 1],
            array_layers: 1,
            mip_levels: 4,
            tiling,
            usage,
            samples: SampleCount::Sample1,
            sharing: Sharing::Exclusive,
            initial_layout: ImageLayout::Undefined,
            ..Default::default()
        };

        let image = vkImage::new(&memory_allocator, info, Default::default())?;

        Ok(image)
    }

    /// Creates a configurable [vkImageview](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageView.html).
    pub(super) fn create_image_view(
        image: Arc<vkImage>,
        format: Format,
        aspects: ImageAspects,
    ) -> Result<Arc<vkImageView>, GraphicsError> {
        let info = ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format,
            subresource_range: ImageSubresourceRange {
                aspects,
                mip_levels: 0..1,
                array_layers: 0..1,
            },
            ..Default::default()
        };

        let view = vkImageView::new(image, info)?;

        Ok(view)
    }
}
