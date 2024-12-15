// --- Modules
pub mod error;
pub(crate) mod ptr;
pub(crate) mod blob;

// --- API Flattening
pub use blob::Blob;

// --- logging and error handling
pub use error::{Error, Result};