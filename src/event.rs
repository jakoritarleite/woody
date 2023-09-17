use std::any::Any;
use std::any::TypeId;

use ahash::HashMap;

use crate::ecs::world::World;

pub trait Event {}

pub trait Handler<E: Event> {
    fn handle(&self, world: &mut World, event: E);
}

impl<E> Handler<E> for fn(&mut World, E)
where
    E: Event,
{
    fn handle(&self, world: &mut World, event: E) {
        self(world, event)
    }
}

#[derive(Debug, Default)]
pub(crate) struct InnerSystem {
    handlers: ErasedStorage,
}

impl InnerSystem {
    pub fn handle<E: Event + 'static>(&mut self, world: &mut World, event: E) {
        let handler = self.handlers.get::<fn(&mut World, E)>();

        if let Some(&handler) = handler {
            handler.handle(world, event);
        }
    }

    pub fn subscribe<E: Event + 'static>(&mut self, handler: fn(&mut World, E)) {
        self.handlers.put(handler);
    }
}

#[derive(Debug, Default)]
struct ErasedStorage {
    items: HashMap<TypeId, Box<dyn Any>>,
}

impl ErasedStorage {
    pub fn put<T: 'static>(&mut self, item: T) {
        self.items.insert(TypeId::of::<T>(), Box::new(item));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let any = self.items.get(&TypeId::of::<T>());
        // Downcast back to concrete type
        any.map(|value| value.downcast_ref::<T>().unwrap())
    }
}

//

#[derive(Debug, Clone, Copy)]
pub struct CreateEvent;
impl Event for CreateEvent {}

#[derive(Debug, Clone, Copy)]
pub struct UpdateEvent;
impl Event for UpdateEvent {}
