use super::archetypes::archetype_from_type_ids;
use super::archetypes::Archetypes;
use super::component::Bundle;
use super::entity::Entities;
use super::entity::Entity;
use super::query::state::QueryState;
use super::query::Query;

/// World is our database
#[derive(Debug, Default)]
pub struct World {
    pub(crate) entities: Entities,
    pub(crate) archetypes: Archetypes,
}

// TODO despawn
impl World {
    pub fn new() -> World {
        World {
            entities: Entities::new(),
            archetypes: Archetypes::new(),
        }
    }

    pub fn spawn<B>(&mut self, bundle: B) -> Entity
    where
        B: Bundle,
    {
        let components_ids = B::components_ids();

        let archetype = archetype_from_type_ids(&components_ids);

        let archetype_storage = self.archetypes.init_storage(archetype, &components_ids);

        let mut entity_row_index = 0;
        bundle.components(archetype_storage, &mut |row_index| {
            entity_row_index = row_index;
        });

        self.entities
            .spawn(entity_row_index, archetype, components_ids)
    }

    pub fn query<Q: Query>(&mut self) -> QueryState<Q> {
        QueryState::new(self)
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::ecs::component::{Bundle, Component};

    use super::World;

    #[derive(Debug)]
    struct Position(i64, i64);

    impl Component for Position {}

    impl Bundle for Position {
        fn components_ids() -> Vec<TypeId> {
            vec![TypeId::of::<Self>()]
        }

        fn components(
            self,
            storage: &mut crate::ecs::archetypes::ArchetypeStorage,
            row_indexes: &mut impl FnMut(usize),
        ) {
            let row_index = storage.init_component(self);

            row_indexes(row_index);
        }
    }

    #[derive(Debug)]
    struct Velocity(u64, u64);

    impl Component for Velocity {}

    impl Bundle for Velocity {
        fn components_ids() -> Vec<TypeId> {
            vec![TypeId::of::<Self>()]
        }

        fn components(
            self,
            storage: &mut crate::ecs::archetypes::ArchetypeStorage,
            row_indexes: &mut impl FnMut(usize),
        ) {
            let row_index = storage.init_component(self);

            row_indexes(row_index);
        }
    }

    #[derive(Debug)]
    struct Health(i8);

    impl Component for Health {}

    impl Bundle for Health {
        fn components_ids() -> Vec<TypeId> {
            vec![TypeId::of::<Self>()]
        }

        fn components(
            self,
            storage: &mut crate::ecs::archetypes::ArchetypeStorage,
            row_indexes: &mut impl FnMut(usize),
        ) {
            let row_index = storage.init_component(self);

            row_indexes(row_index);
        }
    }

    #[test]
    fn spawn_entity() {
        let mut world = World::new();

        world.spawn((Position(0, 0), Velocity(1, 1)));
        world.spawn((Position(0, 0), Velocity(1, 1)));
        world.spawn(Position(0, 0));
        world.spawn((Position(0, 0), Velocity(1, 1)));

        assert!(world.entities.counter == 4);
        assert!(world.entities.entities.len() == 4);
        assert!(world.archetypes.len() == 2);

        let pos_vel_archetype_storage = world.archetypes.get_from_bundle::<(Position, Velocity)>();
        let pos_archetype_storage = world.archetypes.get_from_bundle::<Position>();
        let vel_archetype_storage = world.archetypes.get_from_bundle::<Velocity>();

        assert!(pos_vel_archetype_storage.is_some());
        assert!(pos_archetype_storage.is_some());
        assert!(vel_archetype_storage.is_none());
    }

    #[test]
    fn query_entity_ok() {
        let mut world = World::new();

        world.spawn((Position(10, 200), Velocity(1, 10)));
        world.spawn((Position(-150, 300), Velocity(1, 2)));
        world.spawn(Position(0, 0));
        world.spawn((Position(10, 10), Velocity(2, 1)));
        world.spawn(Velocity(100, 100));
        world.spawn((Position(10, 10), Health(10)));

        let mut query = world.query::<(&Velocity, &mut Position)>();

        let (velocity, mut position) = match query.get(0) {
            Ok((velocity, position)) => (velocity, position),
            _ => panic!("Query that should return Ok returned Err"),
        };

        position.0 += velocity.0 as i64;
        position.1 += velocity.1 as i64;

        assert_eq!(position.0, 11);
        assert_eq!(position.1, 210);
    }

    #[test]
    fn query_entity_err() {
        let mut world = World::new();

        world.spawn((Position(10, 200), Velocity(1, 10)));
        world.spawn((Position(-150, 300), Velocity(1, 2)));
        world.spawn(Position(0, 0));
        world.spawn((Position(10, 10), Velocity(2, 1)));
        world.spawn(Velocity(100, 100));
        world.spawn((Position(10, 10), Health(10)));

        let mut query = world.query::<(&Velocity, &mut Health)>();

        assert!(query.get(0).is_err());

        for i in query.iter() {
            dbg!(i);
        }
    }
}
