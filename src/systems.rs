use crate::ecs::world::World;
use crate::event::Event;
use crate::event::InnerSystem;

#[derive(Debug, Default)]
pub struct Systems {
    inner: InnerSystem,
}

impl Systems {
    pub fn subscribe<E: Event + 'static>(&mut self, handler: fn(&mut World, E)) {
        self.inner.subscribe(handler);
    }

    pub fn fire<E: Event + 'static>(&mut self, world: &mut World, event: E) {
        self.inner.handle(world, event);
    }
}
