use ash::{self, vk};
use std::ffi;
use crate::constants;

pub struct VkState {
    _entry: ash::Entry,
    // instance: ash::Instance,
}


impl VkState {
    pub(crate) fn builder() -> VkStateBuilder { VkStateBuilder::default() }
}



// =============== Builder ===============
pub(crate) struct VkStateBuilder {
    app_name: Option<&'static str>, // ! This will, one way or another, be hardcoded
    app_version: Option<u32>,
    extensions_to_enable: Option<Vec<()>>,
    layers_to_enable: Option<Vec<()>>,
}

impl VkStateBuilder {
    fn default() -> VkStateBuilder {
        VkStateBuilder {
            app_name: None,
            app_version: None,
            extensions_to_enable: None,
            layers_to_enable: None,
        }
    }

    //? Optional Configuration
    pub fn with_app_name(mut self, name: &'static str) -> VkStateBuilder {
        self.app_name = Some(name); self }
    pub fn with_app_version(mut self, version: u32) -> VkStateBuilder { 
        self.app_version = Some(version); self }
    
    //? Build Step
    pub fn build(self) -> Result<VkState, Box<dyn std::error::Error>> {
        // ! Entry
        let entry = ash::Entry::linked();

        // ! ApplicationInfo
        // CString intermediates needed for the ffi with Vulkan
        let engine_name = ffi::CString::new(constants::ENGINE_NAME).unwrap();
        let app_name = ffi::CString::new(
            self.app_name.unwrap_or(constants::DEFAULT_APP_NAME)).unwrap();

        // Use the defaults at 'constants' for the None Options
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(self.app_version.unwrap_or(constants::DEFAULT_APP_VERSION))
            .engine_name(engine_name.as_c_str())
            .engine_version(constants::ENGINE_VERSION)
            .api_version(constants::VK_VERSION)
            .build();
        
        // ! InstanceCreateInfo
        // Append the ApplicationInfo to the InstanceCreateInfo
        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info);
        
        // TODOs
        todo!("VkStateBuilder: Create Vulkan Instance");

        Ok(VkState {
            _entry: entry,
        })
    }
}