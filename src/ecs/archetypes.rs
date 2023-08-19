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
}

impl Archetypes {
    /// Creates a new [`Archetypes`].
    pub(crate) fn new() -> Archetypes {
        Archetypes {
            storages: HashMap::new(),
        }
    }

    /// Returns the Archetype for a certain bundle.
    #[allow(dead_code)]
    pub(crate) fn archetype_from_bundle<B: Bundle>() -> Archetype {
        Self::archetype_from_type_ids(&B::components_ids())
    }

    /// Returns the Archetype for a certain list of TypeId.
    #[inline]
    pub(crate) fn archetype_from_type_ids(type_ids: &[TypeId]) -> Archetype {
        type_ids
            .iter()
            .map(|id| unsafe { std::mem::transmute::<_, u64>(*id) } as u128)
            .sum()
    }

    /// Returns how many archetypes we currently have in the world.
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.storages.len()
    }

    /// Get the [`ArchetypeStorage`] for a certain [`Archetype`].
    pub(crate) fn get(&self, key: Archetype) -> Option<&ArchetypeStorage> {
        self.storages.get(&key)
    }

    /// Initializes an [`ArchetypeStorage`] for a certain [`Archetype`].
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
    /// Creates a new [`ArchetypeStorage`].
    fn new(_components_ids: &[TypeId]) -> ArchetypeStorage {
        ArchetypeStorage {
            components: Components::new(),
        }
    }

    /// Initializes a new component in the storage returning its index in the components storage.
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

    /// Return a reference to the component data from its index.
    ///
    /// As the component may not exist in the storage it returns an Option.
    pub(crate) fn get_component<T: Component>(&self, row_index: usize) -> Option<Ref<'_, T>> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.get(row_index - 1))
    }

    /// Return a mutable reference to the component data from its index.
    ///
    /// As the component may not exist in the storage it returns an Option.
    pub(crate) fn get_component_mut<T: Component>(
        &self,
        row_index: usize,
    ) -> Option<RefMut<'_, T>> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.get_mut(row_index - 1))
    }
}
