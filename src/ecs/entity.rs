use rayon::prelude::ParallelIterator;

use super::archetype::ArchetypeId;
use super::component::ComponentType;

/// Entity stored in our world.
#[derive(Debug, Clone)]
pub struct Entity {
    /// Unique identifier.
    id: u128,
    /// Pointer to the [`Archetype`](crate::world::Archetype) stored in world, which is the key of the archetype in our
    /// `archetypes` [`World`](crate::world::World) field.
    archetype: ArchetypeId,
    /// Pointer to the components index stored in [`ComponentStorage`](crate::component::ComponentStorage).
    //c_ptrs: DashMap<ComponentType, usize, BuildHasherDefault<FxHasher>>,
    c_ptrs: Vec<(ComponentType, usize)>,
}

impl Entity {
    /// Creates a new [`Entity`] with its archetype and component ptrs indexex.
    pub fn new(id: u128, archetype: ArchetypeId, c_ptrs: Vec<(ComponentType, usize)>) -> Self {
        Self {
            id,
            archetype,
            c_ptrs,
        }
    }

    pub(super) unsafe fn c_ptr_unchecked(&self, c_type: ComponentType) -> usize {
        self.c_ptrs
            .iter()
            .find(|(ct, _)| *ct == c_type)
            .map(|(_, index)| *index)
            .unwrap_unchecked()
    }
}
