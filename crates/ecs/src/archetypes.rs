use std::collections::HashMap;
use std::{any::TypeId, collections::hash_map::Entry};

use crate::component::{Component, ComponentStorage, Components};

/// Archetypes are ours tables in the World (database)
#[derive(Debug)]
pub struct Archetypes {
    storages: HashMap<TypeId, ArchetypeStorage>,
}

#[derive(Debug)]
pub struct ArchetypeStorage {
    components: Components,
}

impl Archetypes {
    pub(crate) fn new() -> Archetypes {
        Archetypes {
            storages: HashMap::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.storages.len()
    }

    pub(crate) fn get(&self, key: &TypeId) -> Option<&ArchetypeStorage> {
        self.storages.get(key)
    }

    pub(crate) fn init_storage(&mut self, archetype: &TypeId) -> &mut ArchetypeStorage {
        self.storages
            .entry(*archetype)
            .or_insert(ArchetypeStorage::new())
    }
}

impl ArchetypeStorage {
    fn new() -> ArchetypeStorage {
        ArchetypeStorage {
            components: Components::new(),
        }
    }

    pub fn init_component<T: Component>(&mut self, component: T) -> usize {
        let type_id = TypeId::of::<T>();

        let row_index = match self.components.entry(type_id) {
            Entry::Occupied(mut entry) => {
                let storage = entry.get_mut();

                storage.push(component)
            }
            Entry::Vacant(entry) => {
                let mut storage = ComponentStorage::new();
                let row_index = storage.push(component);

                entry.insert(storage);

                row_index
            }
        };

        row_index
    }
}
