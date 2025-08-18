// --- Modules
pub(crate) mod bundle;
pub mod component;
pub mod entity;
#[macro_use]
pub mod macros;
pub mod resource;
pub mod storage;
pub(crate) mod utils;
pub mod world;

// --- API Flattening
pub use component::Component;
pub use entity::Entity;
pub use resource::Resource;
pub use world::World;
