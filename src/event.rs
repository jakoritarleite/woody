use std::any::Any;
use std::any::TypeId;
use std::collections::hash_map::Entry;

use ahash::HashMap;

use crate::app::GameState;
use crate::ecs::world::World;

pub trait Event: Copy + Send + Sync {}

pub trait Handler<E: Event> {
    fn handle(&self, world: &mut World, state: GameState, event: E);
}

impl<E> Handler<E> for fn(&mut World, GameState, E)
where
    E: Event,
{
    fn handle(&self, world: &mut World, state: GameState, event: E) {
        self(world, state, event)
    }
}

#[derive(Debug, Default)]
pub(crate) struct InnerSystem {
    handlers: ErasedStorage,
}

impl InnerSystem {
    pub fn handle<E: Event + 'static>(&mut self, world: &mut World, state: GameState, event: E) {
        let handlers = self.handlers.get::<fn(&mut World, GameState, E)>();

        if let Some(handlers) = handlers {
            for handler in handlers.iter() {
                handler.handle(world, state, event);
            }
        }
    }

    pub fn subscribe<E: Event + 'static>(&mut self, handler: fn(&mut World, GameState, E)) {
        self.handlers.put(handler);
    }
}

#[derive(Debug, Default)]
struct ErasedStorage {
    items: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl ErasedStorage {
    pub fn put<T: 'static>(&mut self, item: T) {
        match self.items.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut entry) => entry.get_mut().push(Box::new(item)),
            Entry::Vacant(entry) => {
                entry.insert(vec![Box::new(item)]);
            }
        };
    }

    pub fn get<T: 'static>(&self) -> Option<Vec<&T>> {
        let erased = self.items.get(&TypeId::of::<T>());

        if let Some(items) = erased {
            return Some(
                items
                    .iter()
                    .map(|item| item.downcast_ref::<T>().unwrap())
                    .collect(),
            );
        }

        None
    }
}

//

#[derive(Debug, Clone, Copy)]
pub struct CreateEvent;
impl Event for CreateEvent {}

#[derive(Debug, Clone, Copy)]
pub struct UpdateEvent;
impl Event for UpdateEvent {}
