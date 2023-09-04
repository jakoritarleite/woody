use std::fmt;

use nalgebra_glm::vec2;
use nalgebra_glm::vec3;
use thiserror::Error;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::ecs::world::World;
use crate::graphics::context::Graphics;
use crate::graphics::mesh::Rectangle;
use crate::input::keyboard::KeyboardEvent;
use crate::input::CursorEvent;
use crate::input::MouseEvent;
use crate::systems::Systems;

pub struct App {
    pub world: World,
    pub systems: Systems,
    graphics: Graphics,
}

impl App {
    /// Creates a new App.
    pub fn new() -> Result<(Self, EventLoop<()>), Error> {
        let event_loop = EventLoop::new();
        // TODO handle error
        let graphics = Graphics::new(&event_loop).unwrap();
        let mut systems = Systems::new();

        systems.add_draw_system(draw_rectangle);

        Ok((
            Self {
                world: World::new(),
                systems,
                graphics,
            },
            event_loop,
        ))
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        let mut minimized = false;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if !minimized {}

            match event {
                Event::MainEventsCleared if !minimized => {
                    self.systems.run_update_systems(&mut self.world);
                    self.systems
                        .run_draw_systems(&mut self.world, &mut self.graphics);
                    self.graphics.draw().unwrap();

                    // TODO: don't clear meshes
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

                    WindowEvent::KeyboardInput { input, .. } => {
                        let winit::event::KeyboardInput {
                            state,
                            virtual_keycode,
                            ..
                        } = input;

                        if let Some(keycode) = virtual_keycode {
                            let event = KeyboardEvent::new(state, keycode);
                            self.systems
                                .run_keyboard_handler_systems(&mut self.world, event);
                        }
                    }

                    WindowEvent::MouseInput { state, button, .. } => {
                        let event = MouseEvent::new(state, button);

                        self.systems
                            .run_mouse_handler_systems(&mut self.world, event);
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let event = CursorEvent {
                            x: position.x,
                            y: position.y,
                        };

                        self.systems
                            .run_cursor_handler_systems(&mut self.world, event);
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
