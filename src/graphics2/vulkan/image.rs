use ash::vk;
use ash::vk::Extent3D;
use ash::Device;
use ash::Instance;

use super::Error;

/// Abstraction for the Vulkan Image and ImageView.
#[derive(Debug)]
pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub aspect_mask: vk::ImageAspectFlags,
    pub extent: Extent3D,
    pub(super) memory: vk::DeviceMemory,
}

pub struct ImageCreateInfo {
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub aspect_mask: vk::ImageAspectFlags,
    pub extent: Extent3D,
}

impl Image {
    /// Creates a new instance of [`Image`].
    pub fn new(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        device: &Device,
        create_info: ImageCreateInfo,
    ) -> Result<Self, Error> {
        let ImageCreateInfo {
            image_type,
            format,
            tiling,
            usage,
            aspect_mask,
            extent,
        } = create_info;

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

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let image_memory_requirements = unsafe { device.get_image_memory_requirements(image) };

        let memory_type_index = device_memory_properties.memory_types
            [..device_memory_properties.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & image_memory_requirements.memory_type_bits != 0
                    && memory_type.property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL
                        == vk::MemoryPropertyFlags::DEVICE_LOCAL
            })
            .map(|(index, _)| index)
            .ok_or(Error::NoSuitableMemoryIndex)?;

        let allocate_memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirements.size)
            .memory_type_index(memory_type_index as u32);

        let image_memory = unsafe { device.allocate_memory(&allocate_memory_info, None)? };

        unsafe { device.bind_image_memory(image, image_memory, 0)? };

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe { device.create_image_view(&image_view_create_info, None)? };

        Ok(Self {
            image,
            view,
            format,
            tiling,
            usage,
            aspect_mask,
            extent,
            memory: image_memory,
        })
    }
}
