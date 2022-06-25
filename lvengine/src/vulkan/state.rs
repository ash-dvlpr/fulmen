use std::ffi;
use std::os::raw;

use ash::{self, vk, extensions::ext};
use winit::window::Window;
use crate::constants;

pub(crate) struct VkState {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_utils: ext::DebugUtils,
    #[cfg(feature = "validation_layers")]
    debug_call_back: vk::DebugUtilsMessengerEXT,
}


impl VkState {
    pub(crate) fn builder() -> VkStateBuilder { VkStateBuilder::default() }
}

// ! Drop trait implemented to tell Vulkan to free up the memory
impl Drop for VkState {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        println!("Cleaning up Vulkan Resources...");

        unsafe { 
            // ? From the ash crate's examples
            // self.device.device_wait_idle().unwrap();
            // self.device
            //     .destroy_semaphore(self.present_complete_semaphore, None);
            // self.device
            //     .destroy_semaphore(self.rendering_complete_semaphore, None);
            // self.device
            //     .destroy_fence(self.draw_commands_reuse_fence, None);
            // self.device
            //     .destroy_fence(self.setup_commands_reuse_fence, None);
            // self.device.free_memory(self.depth_image_memory, None);
            // self.device.destroy_image_view(self.depth_image_view, None);
            // self.device.destroy_image(self.depth_image, None);
            // for &image_view in self.present_image_views.iter() {
            //     self.device.destroy_image_view(image_view, None);
            // }
            // self.device.destroy_command_pool(self.pool, None);
            // self.swapchain_loader
            //     .destroy_swapchain(self.swapchain, None);
            // self.device.destroy_device(None);
            // self.surface_loader.destroy_surface(self.surface, None);
            #[cfg(feature = "validation_layers")] {
                self.debug_utils
                    .destroy_debug_utils_messenger(self.debug_call_back, None);
                
            }
            self.instance.destroy_instance(None);
        }
    }
}



// =============== Builder ===============
pub(crate) struct VkStateBuilder {
    app_name: Option<&'static str>, // ! This will, one way or another, be hardcoded
    app_version: Option<u32>,
    required_extensions: Vec<*const raw::c_char>,
    #[cfg(feature = "optional_layers")]
    required_layers: Vec<*const raw::c_char>,
}

impl VkStateBuilder {
    fn default() -> Self {
        Self {
            app_name: Some(constants::DEFAULT_APP_NAME),
            app_version: Some(constants::DEFAULT_APP_VERSION),
            required_extensions: Vec::new(),
            #[cfg(feature = "optional_layers")]
            required_layers: Vec::new(),
        }
    }

    //? Optional Configuration
    pub(crate) fn with_app_name(mut self, name: &'static str) -> Self {
        self.app_name = Some(name); self }
    pub(crate) fn with_app_version(mut self, version: u32) -> Self { 
        self.app_version = Some(version); self }

    #[cfg(feature = "validation_layers")] 
    pub(crate) fn with_validation_layers(mut self) -> Self { 
        // Enable the layer and all required extensions
        self.add_layer(constants::vk_validation_layer_name().as_ptr())
            .add_extension(ext::DebugUtils::name().as_ptr())
    }

    #[cfg(feature = "optional_layers")]
    fn add_layer(mut self, layer_name: *const raw::c_char) -> Self {
        self.required_layers.push(layer_name); self }
    fn add_extension(mut self, extension_name_ptr: *const raw::c_char ) -> Self {
        self.required_extensions.push(extension_name_ptr); self }
    
    //? Build Step
    pub(crate) fn build(mut self, window_handle: &Window) -> Result<VkState, Box<dyn std::error::Error>> {
        // ! Entry
        let entry = ash::Entry::linked();

        // ! ApplicationInfo
        // CString intermediates needed for the ffi with Vulkan
        let engine_name = ffi::CString::new(constants::ENGINE_NAME).unwrap();
        let app_name = ffi::CString::new(self.app_name.unwrap()).unwrap();

        // Use the defaults at 'constants' for the None Options
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(self.app_version.unwrap())
            .engine_name(engine_name.as_c_str())
            .engine_version(constants::ENGINE_VERSION)
            .api_version(constants::VK_VERSION)
            .build();
        
        // ! InstanceCreateInfo
        // Append the ApplicationInfo to the InstanceCreateInfo
        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info);

        // Add the window's required extensions to the list of required_extensions
        ash_window::enumerate_required_extensions(window_handle)
            .expect("Failed to enumerate Required Window Extensions")
            .iter()
            .for_each(|ext: &*const raw::c_char| { 
                self.required_extensions.push(*ext);
            });

        // // Extensions for iOS and MacOS
        // #[cfg(any(target_os = "macos", target_os = "ios"))] 
        // {
        //     instance_create_info = instance_create_info
        //         .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        //     self = self
        //         .add_extension(vk::KhrPortabilityEnumerationFn::name().as_ptr())
        //         .add_extension(vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
        // }

        // !  DebugUtilsMessengerCreateInfo
        #[cfg(feature = "validation_layers")] 
        let mut debug_create_info;
        #[cfg(feature = "validation_layers")] {
            debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity( // Severities that trigger the callback
                      vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                )
                .message_type( // Type of messages sent to the callback
                      vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                )
                .pfn_user_callback(Some(super::utils::vulkan_debug_utils_callback))
                .build();

            // Extend the InstanceCreateInfo with the DebugUtilsMessengerCreateInfo
            instance_create_info = instance_create_info.push_next(&mut debug_create_info);
        }
        
        // Append the required layers and extensions to the InstanceCreateInfo
        #[cfg(feature = "optional_layers")] {
            instance_create_info = instance_create_info.enabled_layer_names(&self.required_layers);
        }
        instance_create_info = instance_create_info.enabled_extension_names(&self.required_extensions);


        // TODO : Check that the layers and extensions are supported

        // ! Instance
        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

        // ! DebugUtils
        #[cfg(feature = "validation_layers")]
        let debug_utils;
        #[cfg(feature = "validation_layers")]
        let debug_call_back;
        
        #[cfg(feature = "validation_layers")] {
            debug_utils = ext::DebugUtils::new(&entry, &instance);
            debug_call_back = unsafe {debug_utils.create_debug_utils_messenger(&debug_create_info, None)? };
        }
        
        // ! Physical Device
        // TODO : Physical Device 

        Ok(VkState {
            entry,
            instance,
            #[cfg(feature = "validation_layers")]
            debug_utils,
            #[cfg(feature = "validation_layers")]
            debug_call_back,
        })
    }
}