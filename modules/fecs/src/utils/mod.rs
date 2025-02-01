pub mod ptr;

use std::{any::TypeId, collections::HashMap};
pub type TypeIdMap<V> = HashMap<TypeId, V>;
