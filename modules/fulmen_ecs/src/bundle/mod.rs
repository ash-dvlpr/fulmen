// --- Modules
mod impls;
mod info;

// --- API Flattening
pub use info::*;

use crate::component::{ComponentId, Components};

/// The `Bundle` trait enables [`Components`](crate::component::Component) to be inserted
/// into [`Entities`](crate::entity::Entity) and identifying which ones are part of an Archetype.
///
/// Each bundle represents a static set of [`Component`](crate::component::Component) types, of which there
/// can only be one of each [`Component`](crate::component::Component) type, per bundle.
///
/// # SAFETY:
/// This trait should NEVER be manually implemented. Implementations of this trait MUST
/// only make the call to the callback function once per type, in the same order as the types are defined.
pub unsafe trait Bundle: Send + Sync + 'static {
    /// Calls `id_callback` once per each of this `Bundle`'s [`Components`](crate::component::Component).
    fn get_or_register_component_ids(
        world_components: &mut Components,
        id_callback: &mut impl FnMut(ComponentId),
    );

    /// Calls `id_callback` once per each of this `Bundle`'s [`Components`](crate::component::Component),
    /// passing [`None`] for all unregistered Types.
    fn get_component_ids(
        world_components: &Components,
        id_callback: &mut impl FnMut(Option<ComponentId>),
    );
}
