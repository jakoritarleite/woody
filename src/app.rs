use std::fmt;
use std::time::Duration;
use std::time::Instant;

use thiserror::Error;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::ecs::world::World;
use crate::graphics::renderer::Renderer;
use crate::input::keyboard::KeyboardEvent;
use crate::input::CursorEvent;
use crate::input::MouseEvent;
use crate::systems::Systems;

pub struct App {
    pub world: World,
    pub systems: Systems,
    renderer: Renderer,
    clock: Clock,
    delta_time: f64,
    last_time: f64,
}

impl App {
    /// Creates a new App.
    pub fn new() -> Result<(Self, EventLoop<()>), Error> {
        #[cfg(debug_assertions)]
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let event_loop = EventLoop::new();
        let systems = Systems::new();
        let renderer = Renderer::new(&event_loop).expect("creating renderer frontend");

        Ok((
            Self {
                world: World::new(),
                systems,
                renderer,
                clock: Clock::new(),
                delta_time: 0.0,
                last_time: 0.0,
            },
            event_loop,
        ))
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        let mut minimized = false;

        self.clock.start();
        self.clock.update();

        self.last_time = self.clock.elapsed;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            if !minimized {}

            match event {
                Event::MainEventsCleared if !minimized => {
                    self.clock.update();
                    let current_time = self.clock.elapsed;
                    self.delta_time = current_time - self.last_time;

                    let frame_start_time = Instant::now();

                    self.systems.run_update_systems(&mut self.world);
                    //self.systems
                    //    .run_draw_systems(&mut self.world, &mut self.graphics);
                    // self.graphics.draw().unwrap();

                    self.renderer.draw_frame().unwrap();

                    let _frame_elapsed_time = frame_start_time.elapsed().as_secs_f64();

                    self.last_time = current_time;
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
                            self.renderer.resize().unwrap();
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

impl fmt::Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App").field("world", &self.world).finish()
    }
}

struct Clock {
    start: Instant,
    elapsed: f64,
}

impl Clock {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Duration::new(0, 0).as_secs_f64(),
        }
    }

    fn start(&mut self) {
        self.start = Instant::now();
    }

    fn update(&mut self) {
        self.elapsed = self.start.elapsed().as_secs_f64();
    }
}

#[derive(Debug, Error)]
pub enum Error {
    //#[error("Could not create window: {0}")]
    // Renderer(#[from] graphics::GraphicsError),
}
