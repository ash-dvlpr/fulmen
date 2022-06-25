use crate::constants;
use crate::vulkan::{
    state::VkState,
    // renderer::Renderer,
    // render_system::RenderSystem,
};

use winit::{
    dpi::LogicalSize,
    window::{Window, WindowBuilder}, event_loop::EventLoop,
};

// =======================================
pub struct LveApplication {
    pub window: Window,
    state: VkState,
    // renderer: Renderer,
    // render_system: RenderSystem,
}

impl LveApplication { 
    pub fn builder() -> LveApplicationBuilder { LveApplicationBuilder::default() }
}

// =============== Builder ===============
pub struct LveApplicationBuilder {
    app_name: Option<&'static str>,
    app_version: Option<u32>,
    window_name: Option<String>,
    window_size: Option<LogicalSize<u32>>,
    window_resizable: Option<bool>,
}

impl LveApplicationBuilder {
    fn default() -> Self {
        Self {
            app_name: None,
            app_version: None,
            window_name: Some(constants::DEFAULT_WINDOW_NAME.to_owned()),
            window_size: Some(constants::DEFAULT_WINDOW_SIZE),
            window_resizable: Some(constants::DEFAULT_WINDOW_RESIZABLE),
        }
    }

    //? Optional Configuration
    pub fn with_window_name(mut self, name: &str) -> Self { 
        self.window_name = Some(name.to_owned()); self }
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self { 
        self.window_size = Some(LogicalSize::new(width, height)); self }
    pub fn with_resizable_window(mut self, resizable: bool) -> Self { 
        self.window_resizable = Some(resizable); self }

    //? Build Step
    pub fn build(self) -> (LveApplication, EventLoop<()>) { 
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.window_name.unwrap())
            .with_inner_size(self.window_size.unwrap())
            .with_resizable(self.window_resizable.unwrap())
            .build(&event_loop)
            .expect("Failed to create the window.");

        let mut state_config = VkState::builder();

        // Load in the app's name & version if configured
        if let Some(_name) = self.app_name { state_config = state_config.with_app_name(_name); }
        if let Some(_version) = self.app_version { state_config = state_config.with_app_version(_version); }

        // Enable the vulkan validation layers
        #[cfg(feature = "validation_layers")] {
            state_config = state_config.with_validation_layers(); 
        }

        // Finish building the VkState
        let state = state_config.build(&window).expect("Failed to create the Vulkan Instance.");

        (LveApplication {
            window,
            state,
        }, event_loop)
    }
}