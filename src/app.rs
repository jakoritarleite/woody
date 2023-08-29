use std::fmt;

use nalgebra_glm::vec2;
use nalgebra_glm::vec3;
use thiserror::Error;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::ecs::world::World;
use crate::graphics;
use crate::graphics::context::Graphics;
use crate::graphics::mesh::IntoMesh;
use crate::graphics::mesh::Rectangle;
// use crate::graphics::Renderer;

pub struct App {
    pub world: World,
    #[allow(clippy::type_complexity)]
    update_systems: Vec<Box<dyn Fn(&mut World)>>,
    // TODO create basic mesh components such as Rectangle, Circle, Line, etc
    // and create systems that draws those meshes using our renderer
    #[allow(clippy::type_complexity)]
    draw_systems: Vec<Box<dyn Fn(&mut World, &mut Graphics)>>,
    //
    graphics: Graphics,
}

impl App {
    /// Creates a new App.
    pub fn new() -> Result<(Self, EventLoop<()>), Error> {
        let event_loop = EventLoop::new();
        // TODO handle error
        let graphics = Graphics::new(&event_loop).unwrap();

        Ok((
            Self {
                world: World::new(),
                update_systems: vec![],
                draw_systems: vec![Box::new(draw_rectangle)],
                graphics,
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

        for system in self.draw_systems.iter() {
            (system)(&mut self.world, &mut self.graphics);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        let mut minimized = false;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if !minimized {}

            match event {
                Event::MainEventsCleared if !minimized => {
                    self.run_systems();
                    self.graphics.draw().unwrap();
                    // self.renderer.render().unwrap();
                    // self.renderer.perspective_angle += 1.0;

                    self.graphics.meshes.clear();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        println!("Current frame ( {} )", self.graphics.frame_number);
                    }

                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            minimized = true;
                        } else {
                            minimized = false;
                            self.graphics.recreate_swapchain = true;
                        }
                    }

                    _ => {}
                },

                _ => {}
            }
        });
    }
}

fn draw_rectangle(_world: &mut World, graphics: &mut Graphics) {
    graphics
        .push_mesh(Rectangle::new(vec2(1700.0, 700.0), vec3(0.0, 0.0, 1.0)))
        .unwrap();
    graphics
        .push_mesh(Rectangle::new(vec2(100.0, 410.0), vec3(1.0, 0.0, 0.0)))
        .unwrap();
}

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("world", &self.world)
            .field("renderer", &self.graphics)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    //#[error("Could not create window: {0}")]
    // Renderer(#[from] graphics::GraphicsError),
}
