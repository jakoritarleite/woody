use crate::ecs::world::World;
use crate::graphics::context::Graphics;
use crate::input::keyboard::KeyboardEvent;
use crate::input::CursorEvent;
use crate::input::MouseEvent;

// TODO: implement a better System handler using an event-driven architecture
#[allow(clippy::type_complexity)]
pub struct Systems {
    update_systems: Vec<Box<dyn Fn(&mut World)>>,
    keyboard_handler_systems: Vec<Box<dyn Fn(&mut World, KeyboardEvent)>>,
    mouse_handler_systems: Vec<Box<dyn Fn(&mut World, MouseEvent)>>,
    cursor_handler_systems: Vec<Box<dyn Fn(&mut World, CursorEvent)>>,
    draw_systems: Vec<Box<dyn Fn(&mut World, &mut Graphics)>>,
}

impl Systems {
    pub fn new() -> Self {
        Self {
            update_systems: vec![],
            keyboard_handler_systems: vec![],
            mouse_handler_systems: vec![],
            cursor_handler_systems: vec![],
            draw_systems: vec![],
        }
    }

    pub fn add_system<F>(&mut self, update_system: F)
    where
        F: Fn(&mut World) + 'static,
    {
        self.update_systems.push(Box::new(update_system));
    }

    pub(crate) fn add_draw_system<F>(&mut self, system: F)
    where
        F: Fn(&mut World, &mut Graphics) + 'static,
    {
        self.draw_systems.push(Box::new(system));
    }

    pub fn add_keyboard_event_handler<F>(&mut self, system: F)
    where
        F: Fn(&mut World, KeyboardEvent) + 'static,
    {
        self.keyboard_handler_systems.push(Box::new(system));
    }

    pub fn add_mouse_event_handler<F>(&mut self, system: F)
    where
        F: Fn(&mut World, MouseEvent) + 'static,
    {
        self.mouse_handler_systems.push(Box::new(system));
    }

    pub fn add_cursor_event_handler<F>(&mut self, system: F)
    where
        F: Fn(&mut World, CursorEvent) + 'static,
    {
        self.cursor_handler_systems.push(Box::new(system));
    }

    pub(crate) fn run_update_systems(&self, world: &mut World) {
        for system in self.update_systems.iter() {
            (system)(world);
        }
    }

    pub(crate) fn run_draw_systems(&self, world: &mut World, graphics: &mut Graphics) {
        for system in self.draw_systems.iter() {
            (system)(world, graphics);
        }
    }

    pub(crate) fn run_keyboard_handler_systems(&self, world: &mut World, event: KeyboardEvent) {
        for system in self.keyboard_handler_systems.iter() {
            (system)(world, event);
        }
    }

    pub(crate) fn run_mouse_handler_systems(&mut self, world: &mut World, event: MouseEvent) {
        for system in self.mouse_handler_systems.iter() {
            (system)(world, event);
        }
    }

    pub(crate) fn run_cursor_handler_systems(&mut self, world: &mut World, event: CursorEvent) {
        for system in self.cursor_handler_systems.iter() {
            (system)(world, event);
        }
    }
}

impl Default for Systems {
    fn default() -> Self {
        Self::new()
    }
}
