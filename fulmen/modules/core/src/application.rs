use renderer::VulkanRenderer;

#[derive(Default)]
pub struct AppInfo {
    pub app_name: Option<String>,
    pub app_version: Option<u32>,
    pub resizable_window: bool,
}

#[derive(Default)]
pub struct App {
    appinfo: AppInfo,
    // scene: Scene // Holds entities
    renderer: VulkanRenderer // Handles rendering
}


impl App {
    pub fn new() -> Self {
        Self {
            // TODO: Fix
            renderer: VulkanRenderer::new(),
            ..App::default()
        }
    }

    pub fn with_app_name(&mut self, name: &str) -> &mut Self {
        self.appinfo.app_name = Some(name.to_owned());
        self
    }

    pub fn with_resizable_window(&mut  self, resizable: bool) -> &mut Self {
        self.appinfo.resizable_window = resizable;
        self
    }

    pub fn run(&mut self) {
        // TODO: Init logger
        // TODO: Start event loop
        println!("ENGINE RUN");
    }
}




    