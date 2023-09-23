use std::any::Any;
use std::any::TypeId;
use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rustc_hash::FxHasher;

use super::FxRef;
use super::FxRefMut;

pub use ecs_macros::Component;

pub trait Component: Send + Sync + Any {
    fn component_type() -> ComponentType {
        TypeId::of::<Self>()
    }
}

pub trait Bundle: 'static {
    fn components_types() -> Vec<ComponentType>;

    fn store_components(
        self,
        storages: &mut Components,
        component_indexes: &mut Vec<(ComponentType, usize)>,
    );
}

/// A storage of component storage.
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Components(DashMap<ComponentType, ComponentStorage, BuildHasherDefault<FxHasher>>);

impl Components {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self(DashMap::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::default(),
        ))
    }

    /// Returns how many components we have.
    #[allow(dead_code)]
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets a component storage reference for the [`ComponentType`].
    #[allow(dead_code)]
    pub(super) fn storage(
        &self,
        c_type: ComponentType,
    ) -> Option<FxRef<'_, ComponentType, ComponentStorage>> {
        self.0.get(&c_type)
    }

    /// Gets a storage reference but unwraps the value from Option.
    pub(super) unsafe fn storage_unchecked(
        &self,
        c_type: ComponentType,
    ) -> FxRef<'_, ComponentType, ComponentStorage> {
        self.0.get(&c_type).unwrap_unchecked()
    }

    /// Gets a component storage mutable reference for the [`ComponentType`].
    #[allow(dead_code)]
    pub(super) fn storage_mut(
        &mut self,
        c_type: ComponentType,
    ) -> Option<FxRefMut<'_, ComponentType, ComponentStorage>> {
        self.0.get_mut(&c_type)
    }

    /// Gets a mutable storage reference but unwraps the value from Option.
    pub(super) unsafe fn storage_mut_unchecked(
        &self,
        c_type: ComponentType,
    ) -> FxRefMut<'_, ComponentType, ComponentStorage> {
        self.0.get_mut(&c_type).unwrap_unchecked()
    }

    /// Inserts a new storage into the map and a mutable reference to it. If a storage already
    /// exists returns it.
    pub(super) fn insert(
        &mut self,
        c_type: ComponentType,
    ) -> FxRefMut<'_, ComponentType, ComponentStorage> {
        if self.0.contains_key(&c_type) {
            return unsafe { self.storage_mut_unchecked(c_type) };
        }

        unsafe { self.insert_unchecked(c_type) }
    }

    /// Inserts a new storage into the map but ignores if the entry already exists.
    unsafe fn insert_unchecked(
        &mut self,
        c_type: ComponentType,
    ) -> FxRefMut<'_, ComponentType, ComponentStorage> {
        self.0.insert(c_type, ComponentStorage::from_c_type(c_type));
        self.storage_mut_unchecked(c_type)
    }
}

pub type ComponentType = TypeId;

#[derive(Debug)]
pub struct ComponentPtr {
    #[allow(dead_code)]
    ptr: Box<dyn Any>,
}

impl ComponentPtr {
    /// Creates a new [ComponentPtr] for data.
    pub(super) fn new<T: Component>(data: T) -> Self {
        Self {
            ptr: Box::new(data),
        }
    }

    /// Returns a reference to type T of this pointer.
    ///
    /// If the current pointer is not of type T it'll return None.
    pub(super) fn cast_ref<T: Component>(&self) -> Option<&T> {
        self.ptr.downcast_ref()
    }

    /// Returns a mutable reference to type T of this pointer.
    ///
    /// If the current pointer is not of type T it'll return None.
    pub(super) fn cast_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.ptr.downcast_mut()
    }
}

/// SAFETY: there's any.
unsafe impl Send for ComponentPtr {}

/// SAFETY: there's any.
unsafe impl Sync for ComponentPtr {}

#[derive(Debug)]
pub struct ComponentStorage {
    c_type: ComponentType,
    ptrs: Vec<ComponentPtr>,
}

impl ComponentStorage {
    /// Creates a new [ComponentStorage].
    #[allow(dead_code)]
    pub(super) fn new<T: Component>() -> Self {
        Self::from_c_type(T::component_type())
    }

    /// Creates a new [ComponentStorage] from [ComponentType].
    pub(super) fn from_c_type(c_type: ComponentType) -> Self {
        Self {
            c_type,
            ptrs: Vec::with_capacity(100_000),
        }
    }

    /// Adds a component into this storage and return it's index.
    ///
    /// Note: if the component you're trying to push is not the same type as this storage it won't
    /// be pushed.
    #[allow(dead_code)]
    pub(super) fn push<T: Component>(&mut self, component: T) -> Option<usize> {
        if self.c_type != TypeId::of::<T>() {
            return None;
        }

        // SAFETY: we've already checked if component is the same type as this storage.
        Some(unsafe { self.push_unchecked(component) })
    }

    // TODO: write doc
    pub(super) unsafe fn push_unchecked<T: Component>(&mut self, component: T) -> usize {
        let component = ComponentPtr::new(component);
        self.ptrs.push(component);

        self.ptrs.len() - 1
    }

    /// Returns an iterator over the inner component in this storage.
    #[allow(dead_code)]
    pub(super) fn iter(&self) -> impl ParallelIterator<Item = &ComponentPtr> {
        self.ptrs.par_iter()
    }

    /// Returns an iterator over the inner component in this storage.
    #[allow(dead_code)]
    pub(super) fn iter_mut(&mut self) -> impl ParallelIterator<Item = &mut ComponentPtr> {
        self.ptrs.par_iter_mut()
    }

    /// Gets a component reference from the specified index.
    #[allow(dead_code)]
    pub(super) fn get<T: Component>(&self, index: usize) -> Option<&T> {
        self.ptrs.get(index).and_then(|ptr| ptr.cast_ref::<T>())
    }

    /// Gets a component reference from the specified index but unwraps.
    ///
    /// SAFETY: you must know that the index is valid before calling this method, this way you
    /// assure that the component exists.
    pub(super) unsafe fn get_unchecked<T: Component>(&self, index: usize) -> &T {
        unsafe {
            self.ptrs
                .get(index)
                .unwrap_unchecked()
                .cast_ref::<T>()
                .unwrap_unchecked()
        }
    }

    /// Gets a mutable reference to component from the specified index but unwraps.
    ///
    /// SAFETY: you must know that the index is valid before calling this method, this way you
    /// assure that the component exists.
    pub(super) unsafe fn get_mut_unchecked<T: Component>(&mut self, index: usize) -> &mut T {
        unsafe {
            self.ptrs
                .get_mut(index)
                .unwrap_unchecked()
                .cast_mut::<T>()
                .unwrap_unchecked()
        }
    }
}

impl<T: Component> Bundle for T {
    fn components_types() -> Vec<ComponentType> {
        vec![T::component_type()]
    }

    fn store_components(
        self,
        storages: &mut Components,
        component_indexes: &mut Vec<(ComponentType, usize)>,
    ) {
        // SAFETY: you must guarantee that the storage for this component already exists.
        let mut storage = unsafe { storages.storage_mut_unchecked(T::component_type()) };

        // SAFETY: we now that the storage is the same type of this component in the get above.
        let index = unsafe { storage.push_unchecked(self) };

        component_indexes.push((T::component_type(), index));
    }
}

macro_rules! tuple_impl {
    ( $( $name:ident ),* ) => {
        impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            #![allow(non_snake_case)]

            fn components_types() -> Vec<ComponentType> {
                vec![ $( $name::components_types(), )* ]
                    .iter()
                    .flatten()
                    .map(|c_type| *c_type)
                    .collect()
            }

            fn store_components(
                self,
                storages: &mut Components,
                component_indexes: &mut Vec<(ComponentType, usize)>,
            ) {
                let ($($name,)*) = self;

                $(
                    $name.store_components(storages, component_indexes);
                )*
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
