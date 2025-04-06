// --- Imports
use core::any::TypeId;
use std::collections::HashMap;

// --- Modules
pub mod hash;
pub mod sparse;

// --- API Flattening
pub use sparse::{SparseSetIndex, SparseSet};

/// HashMap that uses the [`TypeId`] directly as a key.
/// As [`TypeId`]s are guaranteed to be unique, this HashMap uses a [`NoOpHash`](`hash::NoOpHash`) to skip re-hashing the key.
pub type TypeIdMap<V> = HashMap<TypeId, V, hash::NoOpHash>;
