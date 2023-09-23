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

    /// Returns the entity id.
    pub fn id(&self) -> u128 {
        self.id
    }

    /// Returns the entity ArchetypeId.
    pub fn archetype_id(&self) -> ArchetypeId {
        self.archetype
    }

    /// Gets the [`ComponentPtr`](crate::ecs::component::ComponentPtr) index in our
    /// [`ComponentStorage`](crate::ecs::component) for the specified [`ComponentType`].
    ///
    /// SAFETY: you must guarantee that this entity has the specified component type.
    pub(super) unsafe fn c_ptr_unchecked(&self, c_type: ComponentType) -> usize {
        self.c_ptrs
            .iter()
            .find(|(ct, _)| *ct == c_type)
            .map(|(_, index)| *index)
            .unwrap_unchecked()
    }
}
