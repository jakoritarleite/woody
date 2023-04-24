use std::any::TypeId;

use crate::archetypes::Archetypes;
use crate::component::Bundle;
use crate::entity::Entities;
use crate::entity::Entity;

/// World is our database
#[derive(Debug)]
pub struct World {
    pub(crate) entities: Entities,
    pub(crate) archetypes: Archetypes,
}

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
        let bundle_id = TypeId::of::<B>();

        let archetype_storage = self.archetypes.init_storage(&bundle_id);

        let mut entity_row_index = 0;
        bundle.components(archetype_storage, &mut |row_index| {
            entity_row_index = row_index;
        });

        self.entities.spawn(entity_row_index)
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::component::{Bundle, Component};

    use super::World;

    #[derive(Debug)]
    struct Position(u64, u64);

    impl Component for Position {}

    impl Bundle for Position {
        fn components(
            self,
            storage: &mut crate::archetypes::ArchetypeStorage,
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
        fn components(
            self,
            storage: &mut crate::archetypes::ArchetypeStorage,
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

        let pos_vel_archetype_storage = world.archetypes.get(&TypeId::of::<(Position, Velocity)>());
        let pos_archetype_storage = world.archetypes.get(&TypeId::of::<Position>());
        let vel_archetype_storage = world.archetypes.get(&TypeId::of::<Velocity>());

        assert!(pos_vel_archetype_storage.is_some());
        assert!(pos_archetype_storage.is_some());
        assert!(vel_archetype_storage.is_none());
    }
}
