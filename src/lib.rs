pub mod app;
pub mod camera;
pub mod ecs;
pub mod event;
#[cfg(not(feature = "graphics2"))]
pub mod graphics;
#[cfg(feature = "graphics2")]
pub mod graphics2;
pub mod input;
mod systems;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
