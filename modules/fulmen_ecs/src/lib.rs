// --- Modules
pub mod macros;
pub mod component;
pub mod resource;
pub mod storage;
pub(crate) mod utils;
pub mod world;
pub mod entity;

// --- API Flattening
pub use component::Component;
pub use resource::Resource;
pub use world::World;
