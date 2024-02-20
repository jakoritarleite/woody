use std::sync::Arc;

use log::info;
use vulkano::command_buffer::sys::RawRecordingCommandBuffer;
use vulkano::command_buffer::BufferImageCopy;
use vulkano::command_buffer::CopyBufferToImageInfo;
use vulkano::command_buffer::CopyImageInfo;
use vulkano::command_buffer::ImageBlit;
use vulkano::command_buffer::ImageCopy;
use vulkano::command_buffer::ResolveImageInfo;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView as vkImageView;
use vulkano::image::view::ImageViewCreateInfo;
use vulkano::image::view::ImageViewType;
use vulkano::image::Image as vkImage;
use vulkano::image::ImageAspects;
use vulkano::image::ImageCreateInfo as vkImageCreateInfo;
use vulkano::image::ImageLayout;
use vulkano::image::ImageSubresourceLayers;
use vulkano::image::ImageSubresourceRange;
use vulkano::image::ImageTiling;
use vulkano::image::ImageType;
use vulkano::image::ImageUsage;
use vulkano::image::SampleCount;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::AccessFlags;
use vulkano::sync::ImageMemoryBarrier;
use vulkano::sync::PipelineStage;
use vulkano::sync::PipelineStages;
use vulkano::sync::QueueFamilyOwnershipTransfer;
use vulkano::sync::Sharing;

use crate::graphics::GraphicsError;

use super::buffer::Buffer;
use super::command_buffer::CommandBuffer;

/// Abstraction of the Vulkan Image and Image view.
#[derive(Debug)]
pub(super) struct Image {
    pub handle: Arc<vkImage>,
    pub view: Arc<vkImageView>,
}

pub(super) struct ImageCreateInfo {
    pub image_type: ImageType,
    pub format: Format,
    pub tiling: ImageTiling,
    pub usage: ImageUsage,
    pub stencil_usage: Option<ImageUsage>,
    pub view_aspects: ImageAspects,
    pub dimensions: [u32; 2],
}

impl Image {
    /// Creates a new instance of [`Image`].
    pub(super) fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        ImageCreateInfo {
            image_type,
            format,
            tiling,
            usage,
            stencil_usage,
            view_aspects,
            dimensions,
        }: ImageCreateInfo,
    ) -> Result<Self, GraphicsError> {
        let image = Self::create_image(
            memory_allocator,
            image_type,
            format,
            tiling,
            usage,
            stencil_usage,
            dimensions,
        )?;
        // TODO: make creating the view configurable ??
        let view = Self::create_image_view(image.clone(), format, view_aspects, usage)?;

        Ok(Self {
            handle: image,
            view,
        })
    }

    /// Creates a configurable [vkImage](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImage.html).
    pub(super) fn create_image(
        memory_allocator: Arc<StandardMemoryAllocator>,
        image_type: ImageType,
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsage,
        stencil_usage: Option<ImageUsage>,
        dimensions: [u32; 2],
    ) -> Result<Arc<vkImage>, GraphicsError> {
        // TODO:
        // - extent:        support configurable depth.
        // - array_layers:  support configurable number of layers in the image.
        // - mip_levels:    support configurable mip mapping.
        // - samples:       support configurable sample count.
        // - sharing:       support configurable sharing mode.
        let info = vkImageCreateInfo {
            image_type,
            format,
            extent: [dimensions[0], dimensions[1], 1],
            array_layers: 1,
            mip_levels: 4,
            tiling,
            usage,
            samples: SampleCount::Sample1,
            sharing: Sharing::Exclusive,
            initial_layout: ImageLayout::Undefined,
            stencil_usage,
            ..Default::default()
        };

        let image = vkImage::new(memory_allocator.clone(), info, Default::default())?;

        Ok(image)
    }

    /// Creates a configurable [vkImageview](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageView.html).
    pub(super) fn create_image_view(
        image: Arc<vkImage>,
        format: Format,
        aspects: ImageAspects,
        usage: ImageUsage,
    ) -> Result<Arc<vkImageView>, GraphicsError> {
        info!(
            "Creating image view with format ({:?}) and aspects ({:?})",
            format, aspects
        );

        let info = ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            format,
            subresource_range: ImageSubresourceRange {
                aspects,
                mip_levels: 0..1,
                array_layers: 0..1,
            },
            usage,
            ..Default::default()
        };

        let view = vkImageView::new(image, info)?;

        Ok(view)
    }

    pub(super) fn _transition_layout(
        &mut self,
        queue: Arc<Queue>,
        _command_buffer: &mut CommandBuffer,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) -> Result<(), GraphicsError> {
        let (src_stage, src_access, dst_stage, dst_access) = match (old_layout, new_layout) {
            // Don't care about the old layout
            // transition to optimal layout (for the underlying implementation).
            (ImageLayout::Undefined, ImageLayout::TransferDstOptimal) => (
                PipelineStage::TopOfPipe,
                AccessFlags::empty(),
                PipelineStage::AllTransfer,
                AccessFlags::TRANSFER_WRITE,
            ),

            (ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal) => (
                PipelineStage::AllTransfer,
                AccessFlags::TRANSFER_WRITE,
                PipelineStage::FragmentShader,
                AccessFlags::SHADER_READ,
            ),

            _ => return Err(GraphicsError::LayoutTransition(old_layout, new_layout)),
        };

        let _barrier = ImageMemoryBarrier {
            src_stages: PipelineStages::from(src_stage),
            src_access,
            dst_stages: PipelineStages::from(dst_stage),
            dst_access,
            old_layout,
            new_layout,
            queue_family_ownership_transfer: Some(
                QueueFamilyOwnershipTransfer::ExclusiveBetweenLocal {
                    src_index: queue.queue_family_index(),
                    dst_index: queue.queue_family_index(),
                },
            ),
            subresource_range: ImageSubresourceRange {
                aspects: ImageAspects::COLOR,
                mip_levels: 0..1,
                array_layers: 0..1,
            },
            ..ImageMemoryBarrier::image(self.handle.clone())
        };

        // TODO: how to use vkCmdPipelineBarrier

        Ok(())
    }

    pub(super) fn copy_from_buffer(
        &mut self,
        // command_buffer_allocator: Arc<StandardMemoryAllocator>,
        command_buffer: &mut CommandBuffer,
        buffer: Buffer<u8>,
    ) -> Result<(), GraphicsError> {
        let region = BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: ImageSubresourceLayers {
                aspects: ImageAspects::COLOR,
                mip_level: 0,
                array_layers: 0..1,
            },
            image_offset: [0; 3],
            image_extent: self.handle.extent(),
            ..Default::default()
        };

        command_buffer
            .handle_mut()?
            .copy_buffer_to_image(CopyBufferToImageInfo {
                dst_image_layout: ImageLayout::TransferDstOptimal,
                regions: [region].into(),
                ..CopyBufferToImageInfo::buffer_image(buffer.handle().clone(), self.handle.clone())
            })?;

        Ok(())
    }
}
