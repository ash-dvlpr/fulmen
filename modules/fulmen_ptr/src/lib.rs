// --- Modules
pub mod ptr;
pub mod util;

// --- API Flattening
pub use ptr::{drop_in_place, get_drop_fn};
pub use ptr::{DropFn, OwnPtr, Ptr, PtrMut};
