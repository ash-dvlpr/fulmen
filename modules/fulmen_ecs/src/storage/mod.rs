// --- Modules
mod resource;

// --- API Flattening
pub use resource::ResourceStorage;

/// The backing data of a [`World`](crate::world::World)
#[derive(Default)]
pub struct Storages {
    pub(crate) resources: ResourceStorage, // Set [ComponentId -> Data]
                                           // TODO: Resource+!Send - Needed for Window resource
}
