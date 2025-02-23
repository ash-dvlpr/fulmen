pub type Error = BlobError;
pub type Result<T> = core::result::Result<T, Error>;

use std::alloc;
use thiserror;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum BlobError {
    #[error("{0}")]
    LayoutError(#[from] alloc::LayoutError),

    #[error("layour of type parameter T doesn't match the Blob's layout")]
    LayoutMistmatch,

    #[error("analagous with Option::None")]
    None,
}
