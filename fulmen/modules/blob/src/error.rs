pub type Error = BlobError;
pub type Result<T> = core::result::Result<T, Error>;

use std::alloc;
use thiserror;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum BlobError {
    #[error("{0}")]
    LayoutError(#[from] alloc::LayoutError),

    #[error("can't create Blob for zero sized types")]
    ZeroSizedLayout,

    #[error("can't safelly create Blob for a type that implements Drop")]
    CantDropType,
}
