use std::collections::HashMap;

use onlyerror::Error;

use crate::ecs::{
    archetypes::archetype_from_type_ids,
    entity::{Entity, EntityDataPointer},
    world::World,
};

use super::Query;

#[derive(Debug)]
pub struct QueryState<'s, Q: Query> {
    pub(crate) query_state: Q::State<'s>,
    pub(crate) entities: HashMap<Entity, EntityDataPointer>,
}

impl<'w, Q: Query> QueryState<'w, Q> {
    pub fn new(world: &'w mut World) -> Self {
        let archetype = archetype_from_type_ids(&Q::component_id());
        let entities: HashMap<_, _> = world
            .entities
            .entities_by_archetype(archetype)
            .into_iter()
            .collect();

        let query_state = Q::init_state(world, archetype);

        Self {
            query_state,
            entities,
        }
    }

    /// Gets the query result for a matched [`Entity`].
    pub fn get(&mut self, entity: Entity) -> Result<Q::Item<'w>, QueryEntityError> {
        let data_pointer = self
            .entities
            .get(&entity)
            .ok_or(QueryEntityError::QueryDoesNotMatch(entity))?;

        Ok(Q::fetch(
            &mut self.query_state,
            (entity, data_pointer.clone()),
        ))
    }

    /// Gets the query result for all matched [`Entity`].
    pub fn iter(&mut self) -> impl Iterator<Item = Q::Item<'w>> + '_ {
        self.entities.iter().map(|(entity, data_pointer)| {
            Q::fetch(&mut self.query_state, (*entity, data_pointer.clone()))
        })
    }
}

#[derive(Debug, Error)]
pub enum QueryEntityError {
    #[error("could not match the Entity {0} for this query")]
    QueryDoesNotMatch(Entity),
}
