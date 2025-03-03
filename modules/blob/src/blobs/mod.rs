// --- Modules
pub mod blob;
mod unsafe_blob;

// --- API Flattening
pub use blob::Blob;
use unsafe_blob::UnsafeBlob;
