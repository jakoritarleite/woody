use ash::vk;
use ash::vk::Extent3D;
use ash::Device;

use super::Error;

/// Abstraction for the Vulkan Image and ImageView.
#[derive(Debug)]
pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub extent: Extent3D,
}

impl Image {
    /// Creates a new instance of [`Image`].
    pub fn new(
        device: &Device,
        image_type: vk::ImageType,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        extent: Extent3D,
    ) -> Result<Self, Error> {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(image_type)
            .format(format)
            .extent(extent)
            .array_layers(1)
            .mip_levels(1)
            .tiling(tiling)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe { device.create_image(&image_create_info, None)? };

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR
                    | vk::ImageAspectFlags::DEPTH
                    | vk::ImageAspectFlags::STENCIL,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe { device.create_image_view(image_view_create_info, None)? };

        Ok(Self {
            image,
            view,
            format,
            tiling,
            usage,
            extent,
        })
    }
}
