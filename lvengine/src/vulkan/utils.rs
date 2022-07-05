use ash::{self, vk, extensions::khr};
use std::ffi;
use std::os::raw;

// ============ FFI Stuff ============
/// External callback function used by the validation layer 
/// TODO : Use a logging crate instead of directly printing to stdout
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
    
    println!("[VULKAN][{}][{}] {}", severity, ty, message.to_string_lossy());
    vk::FALSE
}

// ============== Other ==============
/// Given a slice of c_chars, return a String
pub(crate) fn raw_cstr_to_string(raw_string: &[raw::c_char]) -> String {
    let cstr = unsafe { std::ffi::CStr::from_ptr(raw_string.as_ptr()) };
    cstr.to_string_lossy().into_owned()
}

/// Returns the best PhysicalDevice and the index of it's best queue family
pub(crate) unsafe fn select_physical_device(
    instance: &ash::Instance, 
    surface_loader: &khr::Surface,
    surface: &vk::SurfaceKHR,
) -> Option<(u32, vk::PhysicalDevice)> {
    // Get the list of suitable pdevices
    let pdevices = instance.enumerate_physical_devices().expect("Failed to enumerate Physical Devices");
    // If we have no pysical devices, return None
    if pdevices.is_empty() { return None; }

    // Score & sort all devices. (index, device, score)
    let mut scored_pdevices: Vec<(u32, vk::PhysicalDevice, i32)> = pdevices
        .iter()
        // Score all devices and filter out all devices that couldn't get scored (unsuitable) 
        .filter_map(|pdevice| {
            score_pdevice_with_capabilities(instance, pdevice, surface_loader, surface)
            .map(|(score, index)| { (index, *pdevice, score) })
        })
        .collect();
    scored_pdevices.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    // Return the best scoring one (Last/Biggest)
    Some((
        scored_pdevices.last().unwrap().0,
        scored_pdevices.last().unwrap().1
    ))
}

/// Returns a PhysicalDevice's score along the the index of the best suiting queue family.
/// 
/// Returns None if the PhysicalDevice is incompatible with the Surface or doesn't support Graphics.
unsafe fn score_pdevice_with_capabilities(
    instance: &ash::Instance,
    pdevice: &vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: &vk::SurfaceKHR,
) -> Option<(i32, u32)> {
    let pdevice_properties = instance.get_physical_device_properties(*pdevice);
    let pdevice_queue_families: Vec<(u32, vk::QueueFamilyProperties)> = instance
        .get_physical_device_queue_family_properties(*pdevice)
        .iter().enumerate()
        // Add the queue_family index and convert it to u32 
        .map(|(index, queue_family)| {
            (index as u32, *queue_family)
        })
        // Filter out queue families that either don't support Graphics or the Surface
        .filter(|(index, queue_family_properties)| {
            queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && surface_loader.get_physical_device_surface_support(*pdevice, *index as u32, *surface).unwrap()
        })
        .collect();

    // If there are no suitable queue families, return None
    if pdevice_queue_families.is_empty() { return None; }

    // Score is offsetted by specific properties
    let mut score = match pdevice_properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU   => {  1000 },
        vk::PhysicalDeviceType::INTEGRATED_GPU => { - 250 },
        vk::PhysicalDeviceType::CPU            => { -1000 },
        _ => { 0 }
    };
    // TODO: Score the Queue Families based on capabilities instead of taking the first one
    score += pdevice_properties.limits.max_image_dimension3_d as i32;
    
    let queue_family_index = pdevice_queue_families.first()?.0;

    #[cfg(debug_assertions)] {
        // GPU  [ID] - [Name] - [Score]
        let device_name = raw_cstr_to_string(&pdevice_properties.device_name);
        println!("GPU  [{}] - [{}] - [{}]", &pdevice_properties.device_id, &device_name, &score);          

        // List all family queues that passed the filter
        for queue_family in &pdevice_queue_families {
            println!(" |=> [{}] - Max Queues [{:02}] - Flags [{:?}]", queue_family.0, queue_family.1.queue_count, queue_family.1.queue_flags);
        }
        println!(""); // Separation between info for different GPUs
    }

    Some((score, queue_family_index))
}
