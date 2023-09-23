use super::archetype::Archetype;
use super::archetype::Archetypes;
use super::component::Bundle;
use super::component::Components;
use super::entity::Entity;
use super::query::Query;
use super::query::QueryState;

/// The ECS world where all entities and components will be stored.
#[derive(Debug, Default)]
pub struct World {
    /// All entities stored in this world.
    pub(super) entities: Vec<Entity>,
    /// All entity archetypes stored in this world.
    pub(super) archetypes: Archetypes,
    /// Our storages where which component will go.
    ///
    /// Note: each component has it's own storage.
    pub(super) components: Components,
}

impl World {
    /// Creates a new instance of [`World`].
    pub fn new() -> Self {
        Self {
            entities: Vec::with_capacity(100_000_000),
            archetypes: Archetypes::with_capacity(100_000),
            components: Components::with_capacity(100_000),
        }
    }

    /// Spawns an entity in world.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use mt_ecs::world::World;
    /// use mt_ecs::component::Component;
    ///
    /// struct Position(i64, i64);
    /// struct Velocity(i8);
    ///
    /// impl Component for Position {}
    /// impl Component for Velocity {}
    ///
    /// let mut world = World::new();
    ///
    /// world.spawn((Position(0, 0,), Velocity(10)));
    /// ```
    pub fn spawn<B: Bundle>(&mut self, entity: B) {
        let entity_index = self.entities.len();

        let c_types = B::components_types();

        let archetype_id = Archetype::id_from_c_types(&c_types);
        let mut archetype = self.archetypes.insert(archetype_id, &c_types);

        archetype.entities_mut().push(entity_index);

        // Create a ComponentStorage for each new component.
        for &c_type in c_types.iter() {
            let _ = self.components.insert(c_type);
        }

        let mut c_ptrs = Vec::with_capacity(100);
        entity.store_components(&mut self.components, &mut c_ptrs);

        let entity = Entity::new(entity_index as u128, archetype_id, c_ptrs);

        self.entities.push(entity);
    }

    pub fn query<Q: Query>(&mut self) -> QueryState<'_, Q> {
        QueryState::new(self)
    }
}

mod test {
    use crate::ecs::component::Component;

    #[derive(Debug)]
    struct Position(u8);
    impl Component for Position {}

    #[derive(Debug)]
    struct Velocity(u8);
    impl Component for Velocity {}

    #[test]
    fn spawn() {
        let mut world = super::World::new();

        world.spawn(Position(0));
        world.spawn((Velocity(0), Position(1)));
        world.spawn((Velocity(1), Position(2)));
        world.spawn(Velocity(2));

        // Checks if we have only 2 component storages.
        assert_eq!(world.components.len(), 2);
        // Checks if we have 4 entities.
        assert_eq!(world.entities.len(), 4);
        // Checks if we have 3 archetypes.
        assert_eq!(world.archetypes.len(), 3);
    }

    #[test]
    fn query() {
        let mut world = super::World::new();

        world.spawn(Position(0));
        world.spawn((Velocity(0), Position(1)));
        world.spawn((Velocity(1), Position(2)));
        world.spawn(Velocity(2));

        let mut query = world.query::<(&Velocity, &Position)>();

        for (velocity, pos) in query.iter() {
            println!("{:?}, {:?}", velocity.value(), pos.value());
        }
    }
}
