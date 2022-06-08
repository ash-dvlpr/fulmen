pub(crate) mod constants;
pub mod app;
mod vulkan;

pub use winit;

pub fn hello() {
    println!("Hello Workspaces!");
}