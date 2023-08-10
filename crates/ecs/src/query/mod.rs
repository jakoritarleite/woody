use std::{
    any::TypeId,
    cell::{Ref, RefMut},
};

use crate::{
    archetypes::{Archetype, ArchetypeStorage},
    component::Component,
    entity::{Entity, EntityDataPointer},
    world::World,
};

pub mod state;

pub trait Query {
    /// The item returned by this [`Query`]
    ///
    /// This could be:
    /// - &T
    /// - &mut T
    /// - (&A, &B)
    /// - (&mut A, &B)
    /// - (&mut A, &mut B)
    type Item<'a>;

    /// From where the [`Item`] will be fetched.
    type State<'s>;

    /// Method that returns the TypeId of all Components so that way we can find the Archetype
    fn component_id() -> Vec<TypeId>;

    /// Initiate the [`State`] from the [`World`] and corresponding [`Archetype`]
    fn init_state(world: &World, archetype: Archetype) -> Self::State<'_>;

    /// Fetch the actual [`Item`] of an specific [`Entity`] from the [`State`].
    fn fetch<'s>(
        state: &mut Self::State<'s>,
        entity: (Entity, EntityDataPointer),
    ) -> Self::Item<'s>;
}

impl Query for Entity {
    type Item<'a> = Entity;
    type State<'s> = ();

    fn component_id() -> Vec<TypeId> {
        vec![]
    }

    fn init_state(_world: &World, _archetype: Archetype) -> Self::State<'_> {}

    fn fetch<'s>(
        _state: &mut Self::State<'s>,
        entity: (Entity, EntityDataPointer),
    ) -> Self::Item<'s> {
        entity.0
    }
}

impl<T: Component> Query for &T {
    type Item<'w> = Ref<'w, T>;
    type State<'w> = Option<&'w ArchetypeStorage>;

    fn component_id() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn init_state(world: &World, archetype: Archetype) -> Self::State<'_> {
        world.archetypes.get(archetype)
    }

    fn fetch<'s>(
        state: &mut Self::State<'s>,
        entity: (Entity, EntityDataPointer),
    ) -> Self::Item<'s> {
        // It's safe to unwrap here since the getters will only call this code
        // if there's actually an entity
        state.unwrap().get_component(entity.1.row_index).unwrap()
    }
}

impl<T: Component> Query for &mut T {
    type Item<'w> = RefMut<'w, T>;
    type State<'w> = Option<&'w ArchetypeStorage>;

    fn component_id() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn init_state(world: &World, archetype: Archetype) -> Self::State<'_> {
        world.archetypes.get(archetype)
    }

    fn fetch<'s>(
        state: &mut Self::State<'s>,
        entity: (Entity, EntityDataPointer),
    ) -> Self::Item<'s> {
        // It's safe to unwrap here since the getters will only call this code
        // if there's actually an entity
        state
            .unwrap()
            .get_component_mut(entity.1.row_index)
            .unwrap()
    }
}

macro_rules! query_tuple_impl {
    ( $( $name:ident ),* ) => {
        impl<$($name: Query),*> Query for ($($name,)*) {
            #![allow(non_snake_case)]

            type Item<'a> = ( $( $name::Item<'a>, )* );
            type State<'s> = ( $( $name::State<'s>, )* );

            fn component_id() -> Vec<TypeId> {
                vec![ $( $name::component_id(), )* ]
                    .iter()
                    .flatten()
                    .map(|id| *id)
                    .collect()
            }

            fn init_state(world: & World, archetype: Archetype) -> Self::State<'_> {
                ( $( $name::init_state(world, archetype), )* )
            }

            fn fetch<'s>(state: &mut Self::State<'s>, entity: (Entity, EntityDataPointer)) -> Self::Item<'s> {
                let ($($name,)*) = state;
                ($($name::fetch($name, entity.clone()),)*)
            }
        }
    };
}

macro_rules! query_tuple_impl_all {
    ($head_ty:ident) => {
        query_tuple_impl!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => {
        query_tuple_impl!($head_ty, $( $tail_ty ),*);
        query_tuple_impl_all!( $( $tail_ty ),* );
    };
}

query_tuple_impl_all!(A, B, C, D, E, F, G, H, I, J, K, L);
