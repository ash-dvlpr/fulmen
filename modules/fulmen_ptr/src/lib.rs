// --- Modules
pub mod ptr;
pub mod util;

// --- API Flattening
pub use ptr::{Ptr, PtrMut, OwnPtr, DropFn};
pub use ptr::{drop_in_place, get_drop_fn};
