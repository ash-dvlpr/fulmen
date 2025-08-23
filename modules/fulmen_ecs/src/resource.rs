// Re-export the derive macro for the Trait
pub use fulmen_ecs_macros::Resource;

/// The `Resource` trait allows types to be stored in a [`World`] as a singleton.
///
/// Only one instance of a resource type can exist inside of a [`World`].
pub trait Resource: Send + Sync + 'static {}
