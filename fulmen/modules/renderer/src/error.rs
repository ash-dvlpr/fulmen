use thiserror;

pub type Error = RendererError;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum RendererError {
    #[error("vulkan renderer error `{0}`")]
    VulcanRendererError(#[from] crate::vulkan::Error),
}
