use std::hash::BuildHasherDefault;

use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::MappedRef;
use dashmap::mapref::one::MappedRefMut;
use dashmap::mapref::one::Ref;
use dashmap::mapref::one::RefMut;
use rustc_hash::FxHasher;

pub mod archetype;
pub mod component;
pub mod entity;
pub mod query;
pub mod world;

type FxRef<'a, K, V> = Ref<'a, K, V, BuildHasherDefault<FxHasher>>;
type FxRefMulti<'a, K, V> = RefMulti<'a, K, V, BuildHasherDefault<FxHasher>>;
type FxRefMut<'a, K, V> = RefMut<'a, K, V, BuildHasherDefault<FxHasher>>;
type FxMappedRef<'a, K, V, T> = MappedRef<'a, K, V, T, BuildHasherDefault<FxHasher>>;
type FxMappedRefMut<'a, K, V, T> = MappedRefMut<'a, K, V, T, BuildHasherDefault<FxHasher>>;
