// --- Modules
pub mod error;
pub(crate) mod blobs;

// --- API Flattening
// pub use blobs::Blob;
// pub use blobs::vecblob::VecBlob;

// --- logging and error handling
pub use error::{Error, Result};