mod hash;

use hash::NoOpHash;
use std::{any::TypeId, collections::HashMap};

/// HashMap that uses the [`TypeId`] directly as a key.
/// As [`TypeId`]s are guaranteed to be unique, this HashMap uses a [`NoOpHash`] to skip re-hashing the key.
pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;
