use winit::dpi::PhysicalSize;

pub mod renderer;
pub mod vulkan;

#[derive(Debug, Clone, Copy)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl From<Rgba> for [f32; 4] {
    fn from(value: Rgba) -> Self {
        [value.0, value.1, value.2, value.3]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderArea {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<PhysicalSize<u32>> for RenderArea {
    fn from(value: PhysicalSize<u32>) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: value.width as f32,
            h: value.height as f32,
        }
    }
}

impl From<RenderArea> for ash::vk::Rect2D {
    fn from(value: RenderArea) -> Self {
        Self {
            offset: ash::vk::Offset2D {
                x: value.x as _,
                y: value.y as _,
            },
            extent: ash::vk::Extent2D {
                width: value.w as _,
                height: value.h as _,
            },
        }
    }
}

impl From<(f32, f32, f32, f32)> for RenderArea {
    fn from((x, y, w, h): (f32, f32, f32, f32)) -> Self {
        Self { x, y, w, h }
    }
}
