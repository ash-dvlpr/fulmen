// --- Modules
mod info;

// --- API Flattening
pub use info::*;


/// The `Component` trait allows types to be registered into a [`World`] to be inserted into [`Entities`](crate::entity::Entity).
pub trait Component: Send + Sync + 'static {
    /// A constant indicating the storage type used for this component.
    const STORAGE_TYPE: StorageType;

    // /// Called when registering this component, allowing mutable access to its [`ComponentHooks`].
    // fn register_component_hooks(_hooks: &mut ComponentHooks) {}

    // /// Registers required components.
    // fn register_required_components(
    //     _component_id: ComponentId,
    //     _components: &mut Components,
    //     _storages: &mut Storages,
    //     _required_components: &mut RequiredComponents,
    //     _inheritance_depth: u16,
    // ) {}
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum StorageType {
    #[default]
    Table, // Faster iteration
    SparseSet, // Faster addition and removal
}
