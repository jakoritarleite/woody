use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rustc_hash::FxHasher;

use super::FxRef;
use super::FxRefMulti;
use super::FxRefMut;
use super::component::ComponentType;

/// A storage of archetypes.
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Archetypes(DashMap<ArchetypeId, Archetype, BuildHasherDefault<FxHasher>>);

impl Archetypes {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self(DashMap::with_capacity_and_hasher(capacity, BuildHasherDefault::default()))
    }

    /// Returns how many archetypes we have.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if its empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets an archetype reference from the [`ArchetypeId`].
    #[allow(dead_code)]
    pub(super) fn archetype(&self, id: ArchetypeId) -> Option<FxRef<'_, ArchetypeId, Archetype>> {
        self.0.get(&id)
    }

    /// Gets an archetype reference but unwraps the value from Option.
    ///
    /// SAFETY: you must guarantee that the archetype of the specified id already exists.
    #[allow(dead_code)]
    unsafe fn archetype_unchecked(&self, id: ArchetypeId) -> FxRef<'_, ArchetypeId, Archetype> {
        self.0.get(&id).unwrap_unchecked()
    }

    /// Gets a mutable reference to archetype from the [`ArchetypeId`].
    #[allow(dead_code)]
    pub(super) fn archetype_mut(&mut self, id: ArchetypeId) -> Option<FxRefMut<'_, ArchetypeId, Archetype>> {
        self.0.get_mut(&id)
    }

    /// Gets a mutable archetype reference but unwraps the value from Option.
    ///
    /// SAFETY: you must guarantee that the archetype of the specified id already exists.
    #[allow(dead_code)]
    unsafe fn archetype_mut_unchecked(&mut self, id: ArchetypeId) -> FxRefMut<'_, ArchetypeId, Archetype> {
        self.0.get_mut(&id).unwrap_unchecked()
    }

    /// Inserts a new archetype into the map and a mutable reference to it. If an archetype already exists
    /// it returns a mutable reference to it.
    pub(super) fn insert(&mut self, id: ArchetypeId, c_types: &[ComponentType]) -> FxRefMut<'_, ArchetypeId, Archetype> {
        match self.0.entry(id) {
            Entry::Vacant(entry) => entry.insert(Archetype::new(c_types.to_vec())),
            Entry::Occupied(entry) => entry.into_ref()
        }
    }

    /// Inserts a new archetype into the map but ignores if the entry already exists.
    ///
    /// SAFETY: you must guarantee that the entry does already not exists.
    #[allow(dead_code)]
    pub(super) unsafe fn insert_unchecked(&mut self, id: ArchetypeId, c_types: &[ComponentType]) -> FxRefMut<'_, ArchetypeId, Archetype> {
        self.0.insert(id, Archetype::new(c_types.to_vec()));
        self.archetype_mut_unchecked(id)
    }

    /// Itarates over Archetypes.
    pub(super) fn iter(&self) -> impl ParallelIterator<Item = FxRefMulti<ArchetypeId, Archetype>> {
        self.0.par_iter()
    }

}

/// Unique archetype identifier which is created from the component type list.
pub type ArchetypeId = u128;

/// Archetype is like a type that denotes the components an entity has.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Archetype {
    /// Unique archetype identifier.
    id: ArchetypeId,
    /// Pointer to the Entity stored in world, which is the index of the entity in our `entities`
    /// field [`World`].
    entities: Vec<usize>,
    /// Which components this archetype has.
    c_types: Vec<ComponentType>,
}

impl Archetype {
    /// Creates a new [`Archetype`].
    pub(super) fn new(c_types: Vec<ComponentType>) -> Self {
        Self {
            id: Self::id_from_c_types(&c_types),
            entities: Vec::with_capacity(10_000),
            c_types,
        }
    }

    /// Returns a reference to this archetype entities.
    pub(super) fn entities(&self) -> &Vec<usize> {
        &self.entities
    }

    /// Returns a mutable reference to this archetype entities.
    pub(super) fn entities_mut(&mut self) -> &mut Vec<usize> {
        &mut self.entities
    }

    /// Retrives the ArchetypeId from the component types.
    pub(super) fn id_from_c_types(c_types: &[ComponentType]) -> ArchetypeId {
        c_types
            .iter()
            .map(|c_type| 
                // SAFETY: it's ok to transmute since ComponentType (TypeId) is basically u64.
                unsafe { std::mem::transmute::<ComponentType, u64>(*c_type) } as u128
            )
            .sum()
    }

    /// Checks if this archetype contains certain component types.
    pub(super) fn contains_c_types(&self, c_types: &[ComponentType]) -> bool {
        c_types.par_iter().all(|c_type| self.c_types.contains(c_type))
    }
}

