use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::{any::TypeId, collections::hash_map::Entry};

use super::component::{Bundle, Component, ComponentStorage, Components};

pub type Archetype = u128;

/// Archetypes are ours tables in the World (database)
#[derive(Debug, Default)]
pub struct Archetypes {
    storages: HashMap<Archetype, ArchetypeStorage>,
}

#[derive(Debug, Default)]
pub struct ArchetypeStorage {
    components: Components,
    components_ids: Vec<TypeId>,
}

pub fn archetype_from_type_ids(type_ids: &[TypeId]) -> Archetype {
    type_ids
        .iter()
        .map(|id| unsafe { std::mem::transmute::<_, u64>(*id) } as u128)
        .sum()
}

pub fn archetype_from_bundle<B: Bundle>() -> Archetype {
    archetype_from_type_ids(&B::components_ids())
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

    pub(crate) fn get(&self, key: Archetype) -> Option<&ArchetypeStorage> {
        self.storages.get(&key)
    }

    pub(crate) fn get_from_bundle<B: Bundle>(&self) -> Option<&ArchetypeStorage> {
        let archetype = archetype_from_bundle::<B>();

        self.get(archetype)
    }

    // I still need to use the fucking god
    pub(crate) fn init_storage(
        &mut self,
        archetype: Archetype,
        components_ids: &[TypeId],
    ) -> &mut ArchetypeStorage {
        self.storages
            .entry(archetype)
            .or_insert(ArchetypeStorage::new(components_ids))
    }
}

impl ArchetypeStorage {
    fn new(components_ids: &[TypeId]) -> ArchetypeStorage {
        ArchetypeStorage {
            components: Components::new(),
            components_ids: components_ids.to_vec(),
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

    pub(crate) fn get_component<T: Component>(&self, row_index: usize) -> Option<Ref<'_, T>> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|storage| storage.get(row_index - 1))
            .flatten()
    }

    pub(crate) fn get_component_mut<T: Component>(
        &self,
        row_index: usize,
    ) -> Option<RefMut<'_, T>> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|storage| storage.get_mut(row_index - 1))
            .flatten()
    }
}
