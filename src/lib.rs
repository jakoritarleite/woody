pub mod app;
pub mod ecs;
pub mod event;
pub mod graphics;
pub mod input;
mod systems;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
