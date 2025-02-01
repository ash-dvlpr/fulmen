use ash;
use thiserror;

pub type Error = VulkanRendererError;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum VulkanRendererError {
    #[error("Invalid app name. Should not contain any interior nul bytes {0:?}")]
    InvalidAppName(String),
    #[error("An interior nul byte has been found while building a CString `{0}`")]
    NulError(#[from] std::ffi::NulError),

    #[error("vulkan error `{0}`")]
    VulcanError(#[from] ash::vk::Result),
}
