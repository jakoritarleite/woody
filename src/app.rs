use std::time::Duration;
use std::time::Instant;

use thiserror::Error;

use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::ecs::world::World;
use crate::event::CreateEvent;
use crate::event::UpdateEvent;
use crate::graphics::camera::Camera;
use crate::graphics::renderer::Renderer;
use crate::input::keyboard::KeyboardEvent;
use crate::input::CursorEvent;
use crate::input::MouseEvent;
use crate::input::MouseMotionEvent;
use crate::systems::Systems;

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    pub delta_time: f64,
    pub last_time: f64,
}

pub struct App {
    pub world: World,
    pub systems: Systems,
    renderer: Renderer,
    state: GameState,
    clock: Clock,
}

impl App {
    /// Creates a new App.
    pub fn new() -> Result<(Self, EventLoop<()>), Error> {
        #[cfg(debug_assertions)]
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let world = World::new();

        let event_loop = EventLoop::new()?;
        let systems = Systems::default();
        let renderer = Renderer::new(&event_loop).expect("creating renderer frontend");
        let state = GameState {
            delta_time: 0.0,
            last_time: 0.0,
        };

        Ok((
            Self {
                world,
                systems,
                renderer,
                clock: Clock::new(),
                state,
            },
            event_loop,
        ))
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<(), Error> {
        let mut minimized = false;

        self.systems.fire(&mut self.world, self.state, CreateEvent);

        self.clock.start();
        self.clock.update();

        self.state.last_time = self.clock.elapsed;

        event_loop.run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::AboutToWait => self.renderer.window.request_redraw(),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::RedrawRequested if !minimized => {
                        self.clock.update();
                        let current_time = self.clock.elapsed;
                        self.state.delta_time = current_time - self.state.last_time;

                        let frame_start_time = Instant::now();

                        self.systems.fire(&mut self.world, self.state, UpdateEvent);

                        if let Some(cam) = self.world.query::<&Camera>().iter().next() {
                            self.renderer.set_view(cam.view())
                        };

                        self.renderer.draw_frame().unwrap();

                        let _frame_elapsed_time = frame_start_time.elapsed().as_secs_f64();

                        self.state.last_time = current_time;
                    }

                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    }

                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            minimized = true;
                        } else {
                            minimized = false;
                            self.renderer.resize().unwrap();
                        }
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        let winit::event::KeyEvent {
                            state,
                            physical_key,
                            ..
                        } = event;

                        if let winit::keyboard::PhysicalKey::Code(keycode) = physical_key {
                            let event = KeyboardEvent::new(state, keycode);

                            self.systems.fire(&mut self.world, self.state, event);
                        };
                    }

                    WindowEvent::MouseInput { state, button, .. } => {
                        let event = MouseEvent::new(state, button);

                        self.systems.fire(&mut self.world, self.state, event);
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let event = CursorEvent {
                            x: position.x,
                            y: position.y,
                        };

                        self.systems.fire(&mut self.world, self.state, event);
                    }

                    _ => {}
                },

                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        let delta = (delta.0 as f32, delta.1 as f32);

                        let event = MouseMotionEvent {
                            delta: delta.into(),
                        };

                        self.systems.fire(&mut self.world, self.state, event);
                    }

                    DeviceEvent::MouseWheel { delta: _ } => todo!(),

                    _ => {}
                },

                _ => {}
            }
        })?;

        Ok(())
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
    #[error("event loop failed: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
}
