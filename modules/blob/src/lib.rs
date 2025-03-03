// --- Modules
pub(crate) mod blobs;
pub mod error;

// --- API Flattening
pub use blobs::Blob;
// pub use blobs::vecblob::VecBlob;

// --- logging and error handling
pub use error::{Error, Result};
