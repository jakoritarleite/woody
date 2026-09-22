use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct GlobalUniformObject {
    pub projection: Mat4,
    pub view: Mat4,
}

impl Default for GlobalUniformObject {
    fn default() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
        }
    }
}
