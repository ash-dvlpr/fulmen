use std::ffi;
use ash::{self, vk};


pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
}

impl Drop for VulkanRenderer {
    // Clean up vulkan resources in reverse initialization order.
    fn drop(&mut self) {
        unsafe {
            // TODO: Cleaunp
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanRenderer {
    fn new() -> Self {
        // Load the Ash Vulkan wrapper
        let entry = ash::Entry::linked();

        // AppInfo
        let app_name = unsafe { ffi::CString::new("Fulmen App").unwrap() };
        let engine_name = unsafe { ffi::CString::new("Fulmen").unwrap() };

        let app_info = vk::ApplicationInfo::builder()
            // TODO: configurable app name + version 
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 0, 0, 1))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 0, 0, 1))
            .api_version(vk::API_VERSION_1_2)
            .build();

        // InstanceCreateInfo
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            // TODO: Load extensions
            // [ ]: Required extensions
            // [ ]: Optional extensions
            // [ ]: DEBUG: Validation Layers
            .build();

        // TODO: 
        let instance = unsafe {
            entry.create_instance(&create_info, None).unwrap()
        };

        Self {
            entry,
            instance,
        }
    }
}