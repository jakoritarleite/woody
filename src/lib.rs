pub mod app;
pub mod camera;
pub mod ecs;
pub mod event;
pub mod graphics;
pub mod graphics2;
pub mod input;
mod systems;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
