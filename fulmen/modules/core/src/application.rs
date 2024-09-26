
#[cfg(feature = "logging")]
use log::*;
#[cfg(feature = "rendering")]
use renderer::*;

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

        // TODO: Start event loop

        // Initialize renderer
        #[cfg(feature = "rendering")]
        {
            #[cfg(feature = "logging")]
            info!("Initializing renderer");

            let _result = VulkanRenderer::new();
            self.renderer = if let Err(_error) = _result {
                #[cfg(feature = "logging")]
                error!("- {}", _error);

                None
            } else {
                _result.ok()
            };
        }
    }
}
