use vulkanalia::vk;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::SurfaceKHR;
use vulkanalia::Instance;

use super::RendererError;

#[derive(Debug)]
pub struct SwapchainSupport {
    pub(super) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(super) formats: Vec<vk::SurfaceFormatKHR>,
    pub(super) present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    /// Gets [`SwapchainSupport`] for target physical device.
    pub unsafe fn get(
        instance: &Instance,
        surface: SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self, RendererError> {
        Ok(Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(physical_device, surface)?,
            formats: instance.get_physical_device_surface_formats_khr(physical_device, surface)?,
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(physical_device, surface)?,
        })
    }
}
