use std::{any::TypeId, collections::HashMap};

use crate::archetypes::Archetype;

/// Definition of a entity which is just an identifier
pub type Entity = u64;

/// Collection of entities that are in the world
#[derive(Debug)]
pub struct Entities {
    pub counter: Entity,
    pub entities: HashMap<Entity, EntityDataPointer>,
}

#[derive(Debug, Clone)]
pub struct EntityDataPointer {
    pub(crate) row_index: usize,
    pub(crate) archetype: Archetype,
    pub(crate) components: Vec<TypeId>,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            counter: 0,
            entities: HashMap::new(),
        }
    }

    pub fn spawn(
        &mut self,
        row_index: usize,
        archetype: Archetype,
        components: Vec<TypeId>,
    ) -> Entity {
        let entity = self.counter;
        self.counter += 1;

        self.entities.insert(
            entity,
            EntityDataPointer {
                row_index,
                archetype,
                components,
            },
        );

        entity
    }

    pub fn get(&self, entity: &Entity) -> Option<(Entity, &EntityDataPointer)> {
        self.entities
            .get(entity)
            .map(|data_pointer| (*entity, data_pointer))
    }

    pub fn entities_by_archetype(&self, archetype: Archetype) -> Vec<(Entity, EntityDataPointer)> {
        self.entities
            .iter()
            .filter(|(_, data_pointer)| data_pointer.archetype == archetype)
            .map(|(id, data_pointer)| (*id, data_pointer.clone()))
            .collect()
    }

    pub fn entities_by_component_id(
        &self,
        component_id: &TypeId,
    ) -> Vec<(Entity, EntityDataPointer)> {
        self.entities
            .iter()
            .filter(|(_, data_pointer)| data_pointer.components.contains(component_id))
            .map(|(id, data_pointer)| (*id, data_pointer.clone()))
            .collect()
    }

    pub fn entities_by_components_ids(
        &self,
        components_ids: &[TypeId],
    ) -> Vec<(Entity, EntityDataPointer)> {
        self.entities
            .iter()
            .filter(|(_, data_pointer)| {
                components_ids
                    .iter()
                    .all(|component_id| data_pointer.components.contains(component_id))
            })
            .map(|(id, data_pointer)| (*id, data_pointer.clone()))
            .collect()
    }
}
