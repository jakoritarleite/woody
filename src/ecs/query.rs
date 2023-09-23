use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;

use super::component::Component;
use super::component::ComponentStorage;
use super::component::ComponentType;
use super::component::Components;
use super::entity::Entity;
use super::world::World;
use super::FxMappedRef;
use super::FxMappedRefMut;
use super::FxRef;
use super::FxRefMut;
use std::hash::Hash;

#[repr(transparent)]
#[derive(Debug)]
pub struct MappedRefWrapper<'a, K: Hash + Eq, V, T>(FxMappedRef<'a, K, V, T>);

unsafe impl<'a, K: Hash + Eq, V, T> Send for MappedRefWrapper<'a, K, V, T> {}
unsafe impl<'a, K: Hash + Eq, V, T> Sync for MappedRefWrapper<'a, K, V, T> {}

impl<'a, K: Hash + Eq, V, T> Deref for MappedRefWrapper<'a, K, V, T> {
    type Target = FxMappedRef<'a, K, V, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, K: Hash + Eq, V, T> DerefMut for MappedRefWrapper<'a, K, V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct MappedRefMutWrapper<'a, K: Hash + Eq, V, T>(FxMappedRefMut<'a, K, V, T>);

unsafe impl<'a, K: Hash + Eq, V, T> Send for MappedRefMutWrapper<'a, K, V, T> {}
unsafe impl<'a, K: Hash + Eq, V, T> Sync for MappedRefMutWrapper<'a, K, V, T> {}

impl<'a, K: Hash + Eq, V, T> Deref for MappedRefMutWrapper<'a, K, V, T> {
    type Target = FxMappedRefMut<'a, K, V, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, K: Hash + Eq, V, T> DerefMut for MappedRefMutWrapper<'a, K, V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Query {
    type Item<'a>: Send + Sync;
    type Storage<'a>;
    type ComponentIndex;

    fn components_types() -> Vec<ComponentType>;

    fn init_storage(storages: &Components) -> Self::Storage<'_>;

    fn get_component_index(entity: Entity) -> Self::ComponentIndex;

    fn fetch(storage: Self::Storage<'_>, c_ptr: Self::ComponentIndex) -> Self::Item<'_>;
}

impl<T: Component> Query for &T {
    type Item<'a> = MappedRefWrapper<'a, ComponentType, ComponentStorage, T>;
    type Storage<'a> = FxRef<'a, ComponentType, ComponentStorage>;
    type ComponentIndex = usize;

    fn components_types() -> Vec<ComponentType> {
        vec![T::component_type()]
    }

    fn init_storage(storages: &Components) -> Self::Storage<'_> {
        // SAFETY: there's none.
        unsafe { storages.storage_unchecked(T::component_type()) }
    }

    fn get_component_index(entity: Entity) -> Self::ComponentIndex {
        unsafe { entity.c_ptr_unchecked(T::component_type()) }
    }

    fn fetch(storage: Self::Storage<'_>, c_ptr: Self::ComponentIndex) -> Self::Item<'_> {
        MappedRefWrapper(storage.map(|storage| unsafe { storage.get_unchecked::<T>(c_ptr) }))
    }
}

impl<T: Component> Query for &mut T {
    type Item<'a> = MappedRefMutWrapper<'a, ComponentType, ComponentStorage, T>;
    type Storage<'a> = FxRefMut<'a, ComponentType, ComponentStorage>;
    type ComponentIndex = usize;

    fn components_types() -> Vec<ComponentType> {
        vec![T::component_type()]
    }

    fn init_storage(storages: &Components) -> Self::Storage<'_> {
        // SAFETY: there's none.
        unsafe { storages.storage_mut_unchecked(T::component_type()) }
    }

    fn get_component_index(entity: Entity) -> Self::ComponentIndex {
        unsafe { entity.c_ptr_unchecked(T::component_type()) }
    }

    fn fetch(storage: Self::Storage<'_>, c_ptr: Self::ComponentIndex) -> Self::Item<'_> {
        MappedRefMutWrapper(storage.map(|storage| unsafe { storage.get_mut_unchecked::<T>(c_ptr) }))
    }
}

pub struct QueryState<'a, Q: Query> {
    entities: Vec<&'a Entity>,
    storages: &'a mut Components,
    marker: PhantomData<Q>,
}

impl<'a, Q: Query> QueryState<'a, Q> {
    pub fn new(world: &'a mut World) -> Self {
        let c_types = Q::components_types();

        let archetypes: Vec<_> = world
            .archetypes
            .iter()
            .filter(|archetype| archetype.value().contains_c_types(&c_types))
            .collect();

        let entities: Vec<_> = archetypes
            .par_iter()
            .flat_map(|archetype| {
                archetype
                    .entities()
                    .par_iter()
                    .map(|entity| &world.entities[*entity])
            })
            .collect();

        Self {
            storages: &mut world.components,
            entities,
            marker: PhantomData,
        }
    }

    pub fn par_iter(&mut self) -> impl ParallelIterator<Item = Q::Item<'_>> {
        self.entities.par_iter().map(|&entity| {
            let storage = Q::init_storage(self.storages);
            let c_ptr = Q::get_component_index(entity.clone());
            Q::fetch(storage, c_ptr)
        })
    }

    pub fn iter(&mut self) -> impl Iterator<Item = Q::Item<'_>> {
        self.entities.iter().map(|&entity| {
            let storage = Q::init_storage(self.storages);
            let c_ptr = Q::get_component_index(entity.clone());
            Q::fetch(storage, c_ptr)
        })
    }
}

macro_rules! tuple_impl {
    ( $( $name:ident ),* ) => {
        impl<$($name: Query),*> Query for ($($name,)*) {
            #![allow(non_snake_case)]

            type Item<'a> = ( $( $name::Item<'a>, )* );
            type Storage<'a> = ( $( $name::Storage<'a>, )* );
            type ComponentIndex = ( $( $name::ComponentIndex, )* );

            fn components_types() -> Vec<ComponentType> {
                vec![ $( $name::components_types(), )* ]
                    .iter()
                    .flatten()
                    .map(|c_type| *c_type)
                    .collect()
            }

            fn init_storage(storages: &Components) -> Self::Storage<'_> {
                ( $( $name::init_storage(storages), )* )
            }

            fn get_component_index(entity: Entity) -> Self::ComponentIndex {
                ( $( $name::get_component_index(entity.clone()), )* )
            }

            fn fetch(storage: Self::Storage<'_>, c_ptr: Self::ComponentIndex) -> Self::Item<'_> {
                let ($($name,)*) = storage;
                let ($(paste::paste! { [<$name ptr>] },)*) = c_ptr;

                ( $( $name::fetch($name, paste::paste! { [<$name ptr>] }), )* )
            }
        }
    };
}

tuple_impl!(A);
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
