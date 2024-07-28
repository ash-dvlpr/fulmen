use ash;
use thiserror;

pub type Error = FulmenRendererError;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum FulmenRendererError {
    #[error("app name should not contain any interior nul bytes `{0:?}`")]
    InvalidAppName(String),
    #[error("an interior nul byte has been found while building a CString `{0}`")]
    NulError(#[from] std::ffi::NulError),

    #[error("vulkan error `{0}`")]
    VulcanError(#[from] ash::vk::Result),
}
