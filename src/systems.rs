use crate::app::GameState;
use crate::ecs::world::World;
use crate::event::Event;
use crate::event::InnerSystem;

// TODO: instead of handling all states on flight, create an event buffering that will store all
// "fired" events and always handle them at once.
#[derive(Debug, Default)]
pub struct Systems {
    inner: InnerSystem,
}

impl Systems {
    pub fn subscribe<E: Event + 'static>(&mut self, handler: fn(&mut World, GameState, E)) {
        self.inner.subscribe(handler);
    }

    pub fn fire<E: Event + 'static>(&mut self, world: &mut World, state: GameState, event: E) {
        self.inner.handle(world, state, event);
    }
}
