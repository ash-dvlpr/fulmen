use crate::constants;
use crate::vulkan::state::VkStateBuilder;
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
    window_name: Option<String>,
    window_size: Option<LogicalSize<u32>>,
    window_resizable: Option<bool>,
}

impl LveApplicationBuilder {
    fn default() -> LveApplicationBuilder {
        LveApplicationBuilder {
            window_name: None,
            window_size: None,
            window_resizable: None,
        }
    }

    //? Optional Configuration
    pub fn with_window_name(mut self, name: &str) -> LveApplicationBuilder { 
        self.window_name = Some(name.to_owned()); self }
    pub fn with_window_size(mut self, width: u32, height: u32) -> LveApplicationBuilder { 
        self.window_size = Some(LogicalSize::new(width, height)); self }
    pub fn with_resizable_window(mut self, resizable: bool) -> LveApplicationBuilder { 
        self.window_resizable = Some(resizable); self }

    //? Build Step
    pub fn build(self) -> (LveApplication, EventLoop<()>) { 
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.window_name.unwrap_or(constants::DEFAULT_WINDOW_NAME.to_owned()))
            .with_inner_size(self.window_size.unwrap_or(constants::DEFAULT_WINDOW_SIZE))
            .with_resizable(self.window_resizable.unwrap_or(constants::DEFAULT_WINDOW_RESIZABLE))
            .build(&event_loop)
            .expect("Failed to create the window.");

        let state = VkState::builder()
            .build()
            .expect("Failed to create the Vulkan Instance.");

        (LveApplication {
            window: window,
            state: state,
        }, event_loop)
    }
}