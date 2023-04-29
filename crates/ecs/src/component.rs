use std::any::Any;
use std::any::TypeId;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;

pub use ecs_macros::Component;

use crate::archetypes::Archetype;
use crate::archetypes::ArchetypeStorage;

pub trait Component: Any {}

pub trait Bundle: 'static {
    fn components_ids() -> Vec<TypeId>;

    fn components(self, storage: &mut ArchetypeStorage, row_indexes: &mut impl FnMut(usize));
}

pub type Components = HashMap<TypeId, ComponentStorage>;

#[derive(Debug)]
pub struct ComponentStorage {
    data: Vec<ErasedComponentData>,
}

#[derive(Debug)]
pub struct ErasedComponentData(Rc<RefCell<dyn Any>>);

impl ComponentStorage {
    pub(crate) fn new() -> ComponentStorage {
        ComponentStorage { data: vec![] }
    }

    pub(crate) fn push<T: Component>(&mut self, component: T) -> usize {
        self.data.push(ErasedComponentData::new(component));

        self.data.len()
    }

    pub(crate) fn get<T: Component>(&self, row_index: usize) -> Option<Ref<'_, T>> {
        self.data
            .get(row_index)
            .map(|erased_data| erased_data.cast_ref())
            .flatten()
    }

    pub(crate) fn get_mut<T: Component>(&self, row_index: usize) -> Option<RefMut<'_, T>> {
        self.data
            .get(row_index)
            .map(|erased_data| erased_data.cast_mut())
            .flatten()
    }
}

impl ErasedComponentData {
    pub(crate) fn new<T: Component>(data: T) -> ErasedComponentData {
        let data = Rc::new(RefCell::new(data));
        ErasedComponentData(data)
    }

    pub(crate) fn cast_ref<T: Component>(&self) -> Option<Ref<'_, T>> {
        downcast_ref::<T>(&self.0)
    }

    pub(crate) fn cast_mut<T: Component>(&self) -> Option<RefMut<'_, T>> {
        downcast_mut::<T>(&self.0)
    }
}

fn downcast_ref<'w, T: Any>(cell: &'w RefCell<dyn Any>) -> Option<Ref<'w, T>> {
    let r = cell.borrow();

    if (*r).type_id() == TypeId::of::<T>() {
        return Some(Ref::map(r, |x| x.downcast_ref::<T>().unwrap()));
    }

    None
}

fn downcast_mut<'w, T: Any>(cell: &'w RefCell<dyn Any>) -> Option<RefMut<'w, T>> {
    let r = cell.borrow_mut();

    if (*r).type_id() == TypeId::of::<T>() {
        return Some(RefMut::map(r, |x| x.downcast_mut::<T>().unwrap()));
    }

    None
}

macro_rules! tuple_impls {
    ($head_ty:ident) => {
        tuple_impl!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => {
        tuple_impl!($head_ty, $( $tail_ty ),*);
        tuple_impls!($( $tail_ty ),*);
    };
}

macro_rules! tuple_impl {
    ( $( $name:ident ),* ) => {
        impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            #![allow(non_snake_case)]

            fn components_ids() -> Vec<TypeId> {
                vec![ $( $name::components_ids(), )* ]
                    .iter()
                    .flatten()
                    .map(|id| *id)
                    .collect()
            }

            fn components(
                self,
                storage: &mut ArchetypeStorage,
                row_indexes: &mut impl FnMut(usize)
            ) {
                let ($($name,)*) = self;

                $(
                    $name.components(storage, row_indexes);
                )*
            }
        }
    };
}

tuple_impls!(A, B, C, D, E, F, G, H, I, J, K, L);
