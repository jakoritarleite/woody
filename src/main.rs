use ecs_macros::Component;
use glam::vec3;
use glam::Quat;
use glam::Vec3;
use rayon::prelude::ParallelIterator;
use woody::app::App;
use woody::app::GameState;
use woody::ecs::world::World;
use woody::event::CreateEvent;
use woody::event::UpdateEvent;
use woody::graphics::camera::Camera;
use woody::input::keyboard::KeyCode;
use woody::input::keyboard::KeyboardEvent;
use woody::input::keyboard::KeyboardState;
use woody::input::MouseButton;
use woody::input::MouseEvent;
use woody::input::MouseMotionEvent;
use woody::input::MouseState;

#[derive(Debug, Component)]
pub struct Position(f64, f64, f64);

#[derive(Debug, Component)]
pub struct Velocity(u8, u8);

#[derive(Debug, Component)]
pub struct Health(u8);

fn main() {
    let (mut app, event_loop) = App::new().unwrap();

    app.systems.subscribe(setup);
    // app.systems.subscribe(positions);
    app.systems.subscribe(handle_player_movement);
    app.systems.subscribe(handle_camera_movement);
    // app.systems.subscribe(handle_shot);

    app.run(event_loop).unwrap();
}

fn setup(world: &mut World, _: GameState, _: CreateEvent) {
    world.spawn(Camera {
        position: vec3(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
    });

    world.spawn((Position(0.0, 0.0, -30.0), Velocity(5, 0)));
}

fn positions(world: &mut World, state: GameState, _: UpdateEvent) {
    let mut query = world.query::<(&mut Position, &Velocity)>();

    query.par_iter().for_each(|(mut position, velocity)| {
        position.0 += velocity.0 as f64 * state.delta_time;
        position.1 += velocity.1 as f64 * state.delta_time;
    });
}

fn handle_player_movement(world: &mut World, state: GameState, event: KeyboardEvent) {
    static SPEED: f32 = 25.0;

    let mut query = world.query::<&mut Camera>();

    query.par_iter().for_each(|mut cam| {
        let mut velocity = Vec3::ZERO;

        if event.state == KeyboardState::Released {
            return;
        }

        velocity += match event.keycode {
            KeyCode::KeyW => cam.forward(),
            KeyCode::KeyS => cam.backward(),
            KeyCode::KeyA => cam.left(),
            KeyCode::KeyD => cam.right(),

            _ => vec3(0.0, 0.0, 0.0),
        };

        if !velocity.abs_diff_eq(Vec3::ZERO, 0.0002) {
            cam.position += velocity * SPEED * state.delta_time as f32;
        }
    });
}

fn handle_camera_movement(world: &mut World, state: GameState, event: MouseMotionEvent) {
    static SENSITIVITY: f32 = 0.08;

    let MouseMotionEvent { delta } = event;

    let mut query = world.query::<&mut Camera>();

    query.par_iter().for_each(|mut cam| {
        cam.yaw(-delta.x * SENSITIVITY * state.delta_time as f32);
        cam.pitch(delta.y * SENSITIVITY * state.delta_time as f32);
    });
}

fn handle_shot(_world: &mut World, _: GameState, event: MouseEvent) {
    println!("Just pressed mouse button: {:?}", event);

    match event.state {
        MouseState::Pressed if event.button == MouseButton::Left => {
            // println!("Just pressed Left mouse button");
        }
        _ => {}
    }
}

#[inline]
pub fn clamp(input: f32, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
