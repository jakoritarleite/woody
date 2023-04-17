use std::collections::HashMap;

/// Definition of a entity which is just an identifier
pub type Entity = u64;

/// Collection of entities that are in the world
#[derive(Debug)]
pub struct Entities {
    pub counter: Entity,
    pub entities: HashMap<Entity, EntityDataPointer>,
}

#[derive(Debug)]
pub struct EntityDataPointer {
    row_index: usize,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            counter: 0,
            entities: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, row_index: usize) -> Entity {
        let entity = self.counter;
        self.counter += 1;

        self.entities
            .insert(entity, EntityDataPointer { row_index });

        entity
    }
}
