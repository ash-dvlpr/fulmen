use ash::vk;
use std::ffi;
use winit::dpi::LogicalSize;

// ? Defaults
pub(crate) const DEFAULT_WINDOW_NAME: &str = "LveEngine App";
pub(crate) const DEFAULT_WINDOW_SIZE: LogicalSize<u32> = LogicalSize {width: 600, height: 500};
pub(crate) const DEFAULT_WINDOW_RESIZABLE: bool = true;

pub(crate) const DEFAULT_APP_NAME: &str = "LVE Application";
pub(crate) const DEFAULT_APP_VERSION: u32 = vk::make_api_version(0, 0, 0, 1);

pub(crate) const ENGINE_NAME: &str = "Little Vulkan Engine";
pub(crate) const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 1);
pub(crate) const VK_VERSION: u32 = vk::API_VERSION_1_3;


// ? Alvaliable Layers and Extensions
// Layers
#[cfg(feature = "validation_layers")]
pub(crate) const fn vk_validation_layer_name() -> &'static ffi::CStr {
    unsafe { ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }
}