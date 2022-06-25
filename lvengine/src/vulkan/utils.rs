use ash::vk;
use std::ffi;

// ============ FFI Stuff ============
/// External callback function used by the validation layer 
/// TODO : Use a logging crate instead of directly 
#[cfg(feature = "validation_layers")]
pub(crate) unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT, // Severities that trigger the callback
    message_type: vk::DebugUtilsMessageTypeFlagsEXT, // Types of events that trigger the callback
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT, // Data returned to the callback
    _p_user_data: *mut ffi::c_void,
) -> vk::Bool32 {
    let message = ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[VULKAN][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

// ============== Other ==============
