use std::sync::Arc;

use bitflags::bitflags;

use ash;
use ash::vk;

use super::Error;

/// Abstraction of a [VkFence](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFence.html).
pub struct Fence {
    pub(super) handle: vk::Fence,
    flags: FenceCreateFlags,
    _device: Arc<ash::Device>,
}

impl Fence {
    /// Creates a new instance of [`Fence`].
    pub fn new(device: Arc<ash::Device>, flags: FenceCreateFlags) -> Result<Self, Error> {
        let create_info = vk::FenceCreateInfo::builder().flags(flags.into());

        let fence = unsafe { device.create_fence(&create_info, None)? };

        Ok(Self {
            handle: fence,
            flags,
            _device: device,
        })
    }

    /// Destroys the inner VkFence.
    pub fn destroy(&mut self) {
        unsafe {
            self._device.destroy_fence(self.handle, None);
        }
    }

    /// Waits on a fence with the specified timeout in nanoseconds.
    pub fn wait(&self, timeout: u64) -> Result<bool, Error> {
        if self.is_signaled()? {
            return Ok(true);
        }

        let result = unsafe { self._device.wait_for_fences(&[self.handle], true, timeout) };

        if let Err(error) = result {
            match error {
                vk::Result::TIMEOUT => {
                    log::error!("Fence {:?} timed out", self.handle);
                }

                vk::Result::ERROR_DEVICE_LOST => {
                    log::error!(
                        "Fence {:?} lost device {:?}",
                        self.handle,
                        self._device.handle()
                    );
                }

                vk::Result::ERROR_OUT_OF_HOST_MEMORY => {
                    log::error!("Fence {:?} is out of host memory", self.handle);
                }

                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
                    log::error!("Fence {:?} is out of host memory", self.handle);
                }

                _ => {}
            }

            return Err(error)?;
        }

        Ok(false)
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        unsafe { self._device.reset_fences(&[self.handle])? };

        Ok(())
    }

    /// Checks if this fence is signaled.
    pub fn is_signaled(&self) -> Result<bool, Error> {
        let status = unsafe { self._device.get_fence_status(self.handle)? };

        Ok(status)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FenceCreateFlags(u32);

bitflags! {
    impl FenceCreateFlags: u32 {
        const Signaled = vk::FenceCreateFlags::SIGNALED.as_raw();
    }
}

impl From<FenceCreateFlags> for vk::FenceCreateFlags {
    fn from(value: FenceCreateFlags) -> Self {
        Self::from_raw(value.0)
    }
}
