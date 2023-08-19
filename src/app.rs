use std::fmt;

use thiserror::Error;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::ecs::world::World;
use crate::graphics;
use crate::graphics::Renderer;

pub struct App {
    pub world: World,
    #[allow(clippy::type_complexity)]
    update_systems: Vec<Box<dyn Fn(&mut World)>>,
    // TODO create basic mesh components such as Rectangle, Circle, Line, etc
    // and create systems that draws those meshes using our renderer
    //
    // draw_systems: Vec<Box<dyn Fn(&mut World, &mut Renderer)>>
    renderer: Renderer,
}

impl App {
    /// Creates a new App.
    pub fn new() -> Result<(Self, EventLoop<()>), Error> {
        let event_loop = EventLoop::new();
        let renderer = match Renderer::new(&event_loop) {
            Ok(renderer) => renderer,
            Err(err) => {
                dbg!(&err);
                return Err(Error::Renderer(err));
            }
        };

        Ok((
            Self {
                world: World::new(),
                update_systems: vec![],
                renderer,
            },
            event_loop,
        ))
    }

    pub fn add_system<F>(&mut self, update_system: F)
    where
        F: Fn(&mut World) + 'static,
    {
        self.update_systems.push(Box::new(update_system));
    }

    fn run_systems(&mut self) {
        for system in self.update_systems.iter() {
            (system)(&mut self.world);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        let mut minimized = false;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared if !minimized => {
                    self.run_systems();
                    self.renderer.render().unwrap();
                    self.renderer.perspective_angle += 1.0;
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            minimized = true;
                        } else {
                            minimized = false;
                            self.renderer.recreate_swapchain = true;
                        }
                    }

                    _ => {}
                },

                _ => {}
            }
        });
    }
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("world", &self.world)
            .field("renderer", &self.renderer)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not create window: {0}")]
    Renderer(#[from] graphics::RendererError),
}
