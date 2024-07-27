use ash::{self, ext::debug_utils, vk};
use std::ffi;
use std::os::raw;

#[cfg(feature = "logging")]
use log::*;

pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,

    #[cfg(feature = "vk_validation")]
    debug_utils_loader: debug_utils::Instance,
    #[cfg(feature = "vk_validation")]
    debug_callback: vk::DebugUtilsMessengerEXT,
}

impl Drop for VulkanRenderer {
    // Clean up vulkan resources in reverse initialization order.
    fn drop(&mut self) {
        unsafe {
            // TODO:

            // TODO: Cleaunp
            #[cfg(feature = "vk_validation")]
            {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_callback, None);
            }

            self.instance.destroy_instance(None);
        }
    }

    // fn drop(&mut self) {
    //     unsafe {
    //         self.device.device_wait_idle().unwrap();
    //         self.device
    //             .destroy_semaphore(self.present_complete_semaphore, None);
    //         self.device
    //             .destroy_semaphore(self.rendering_complete_semaphore, None);
    //         self.device
    //             .destroy_fence(self.draw_commands_reuse_fence, None);
    //         self.device
    //             .destroy_fence(self.setup_commands_reuse_fence, None);
    //         self.device.free_memory(self.depth_image_memory, None);
    //         self.device.destroy_image_view(self.depth_image_view, None);
    //         self.device.destroy_image(self.depth_image, None);
    //         for &image_view in self.present_image_views.iter() {
    //             self.device.destroy_image_view(image_view, None);
    //         }
    //         self.device.destroy_command_pool(self.pool, None);
    //         self.swapchain_loader
    //             .destroy_swapchain(self.swapchain, None);
    //         self.device.destroy_device(None);
    //         self.surface_loader.destroy_surface(self.surface, None);

    //         self.debug_utils_loader
    //             .destroy_debug_utils_messenger(self.debug_call_back, None);
    //         self.instance.destroy_instance(None);
    //     }
    // }
}

impl VulkanRenderer {
    pub fn new() -> Self {
        // ? Load the Ash Vulkan wrapper
        let entry = ash::Entry::linked();

        // AppInfo
        let app_name = ffi::CString::new("Fulmen App").unwrap();
        let engine_name = ffi::CString::new("Fulmen").unwrap();

        let app_info = vk::ApplicationInfo::default()
            // TODO: configurable app name + version
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 0, 0, 1))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 0, 0, 1))
            .api_version(vk::API_VERSION_1_3);

        // InstanceCreateInfo
        let mut instance_create_info =
            vk::InstanceCreateInfo::default().application_info(&app_info);

        // ? Enumerate and enable the required extensions and layers
        let mut selected_extensions: Vec<*const raw::c_char> = vec![];
        let mut selected_layers: Vec<*const raw::c_char> = vec![];

        // Debug Utils Messenger
        #[cfg(feature = "vk_validation")]
        {
            // Required extensions and layers
            use crate::vulkan::vk::EXT_DEBUG_UTILS_NAME;
            selected_extensions.push(EXT_DEBUG_UTILS_NAME.as_ptr());
            selected_layers.push(vk_layer_khronos_validation().as_ptr());

            // You could extend the InstanceCreateInfo with the DebugUtilsMessengerCreateInfo
            // This will send messages about preloaded layers from other software intalled on your computer
            // instance_create_info = instance_create_info.push_next(&mut debug_create_info);
        };

        // Select optional extensions based on context
        // TODO: MacOS specific extensions (+create_info.flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR))
        // TODO: Window's required extensions
        // ash_window::enumerate_required_extensions(window_handle)?.iter()
        //         .for_each(|ext: &*const raw::c_char| {
        //             self.required_extensions.push(*ext);
        //         });

        // ? Append the selected extensions and layers to the InstanceCreateInfo
        instance_create_info = instance_create_info
            .enabled_layer_names(&selected_layers)
            .enabled_extension_names(&selected_extensions);

        // ? Create the Instance
        let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

        // ? Debug Utils
        #[cfg(feature = "vk_validation")]
        let debug_utils_loader;
        #[cfg(feature = "vk_validation")]
        let debug_callback;
        #[cfg(feature = "vk_validation")]
        {
            let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vk_validation_debug_utils_callback));

            debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
            debug_callback = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(&debug_create_info, None)
                    .unwrap()
            };
        };

        // ? Create Window Surface

        // Return
        Self {
            entry,
            instance,

            #[cfg(feature = "vk_validation")]
            debug_utils_loader,
            #[cfg(feature = "vk_validation")]
            debug_callback,
        }
    }
}

/// Callback method called by the vulkan validation layers' debug utils
#[cfg(feature = "vk_validation")]
unsafe extern "system" fn vk_validation_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let message_type = format!("{:?}", message_type).to_lowercase();

    let log_level: Option<log::Level> = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => Some(log::Level::Debug),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => Some(log::Level::Info),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => Some(log::Level::Warn),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => Some(log::Level::Error),
        _ => None,
    };

    if let Some(level) = log_level {
        // println!("[VK Debug][{}][{}] {:?}", severity, ty, message);
        log!(target: "VK Debug", level, "[{}] {:?}", message_type, message);
    }

    vk::FALSE
}

#[cfg(feature = "vk_validation")]
pub(crate) const fn vk_layer_khronos_validation() -> &'static ffi::CStr {
    unsafe { ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }
}
