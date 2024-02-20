use std::ops::Deref;
use std::sync::Arc;

use ash::vk;

use super::device::PhysicalDevice;
use super::Error;

/// Abstraction of [VkInstance](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstance.html).
pub struct Instance {
    entry: ash::Entry,
    handle: ash::Instance,

    create_info: vk::InstanceCreateInfo,
    physical_devices: Vec<Arc<PhysicalDevice>>,
}

impl Instance {
    /// Creates a new instance of [`Instance`].
    pub fn new(entry: ash::Entry, create_info: &vk::InstanceCreateInfo) -> Result<Self, Error> {
        let instance = unsafe { entry.create_instance(create_info, None)? };

        let physical_devices = unsafe { instance.enumerate_physical_devices()? }
            .into_iter()
            .map(|pd| PhysicalDevice::from_handle(&instance, pd))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect();

        Ok(Self {
            entry,
            handle: instance,

            create_info: *create_info,
            physical_devices,
        })
    }

    /// Returns an iterator over the instance physical devices.
    pub fn enumerate_physical_devices(&self) -> impl Iterator<Item = Arc<PhysicalDevice>> + '_ {
        self.physical_devices.iter().cloned()
    }

    /// Returns a reference to the inner ash instance.
    pub fn handle(&self) -> &ash::Instance {
        &self.handle
    }
}

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

impl Deref for Instance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
