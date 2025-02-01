// --- std
use std::rc::Rc;

// --- logging and error handling
#[cfg(feature = "logging")]
use log::*;

// --- rendering
#[cfg(feature = "rendering")]
use renderer::*;
#[cfg(feature = "rendering")]
use winit::event_loop::{EventLoop, EventLoopBuilder};

#[derive(Default)]
pub struct AppInfo {
    pub app_name: Option<String>,
    pub app_version: Option<u32>,
    pub resizable_window: bool,
}

#[derive(Default)]
pub struct App {
    appinfo: AppInfo,

    // world: ECS world
    // resources: ECS resources
    // plugins: App plugins

    // renderer may be optional if building the engine without a render feature
    #[cfg(feature = "rendering")]
    renderer: Option<VulkanRenderer>, // Handles rendering
    #[cfg(feature = "rendering")]
    event_loop: Option<Rc<EventLoop<()>>>, // Used for
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_app_name(&mut self, name: &str) -> &mut Self {
        self.appinfo.app_name = Some(name.to_owned());
        self
    }

    pub fn with_resizable_window(&mut self, resizable: bool) -> &mut Self {
        self.appinfo.resizable_window = resizable;
        self
    }

    pub fn run(&mut self) {
        #[cfg(feature = "logging")]
        info!("Running Application");

        // Initialize renderer
        #[cfg(feature = "rendering")]
        {
            #[cfg(feature = "debug_logging")]
            trace!("- Trying to build EventLoop");
            // let _result = build_event_loop::<()>().build();
            self.event_loop = match build_event_loop::<()>().build() {
                Ok(_loop) => Some(Rc::new(_loop)),
                Err(_error) => {
                    #[cfg(feature = "logging")]
                    error!("- {}", _error);
                    None
                }
            };


            let _result = VulkanRenderer::new();
            self.renderer = if let Err(_error) = _result {
                #[cfg(feature = "logging")]
                error!("- {}", _error);

                None
            } else {
                _result.ok()
            };
        }

        // TODO: Start event loop
    }
}

fn build_event_loop<T>() -> EventLoopBuilder<T> {
    #[allow(unused_mut)]
    let mut event_loop_builder = EventLoop::<T>::with_user_event();

    // TODO: Maybe need to set up some stuff using one of these extensions:
    // use winit::platform::x11::EventLoopBuilderExtX11;
    // use winit::platform::wayland::EventLoopBuilderExtWayland;
    // use winit::platform::windows::EventLoopBuilderExtWindows;

    // NOTE: There are also Window extensions
    // use winit::platform::windows::WindowAttributesExtWindows;

    event_loop_builder
}
