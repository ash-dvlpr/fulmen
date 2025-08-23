use fulmen_ptr::OwnPtr;

use crate::component::{Component, ComponentId, Components};
use crate::entity::Entities;
use crate::resource::Resource;
use crate::storage::Storages;
use crate::bundle::Bundles;

mod tests;

pub struct World {
    /// Manages all the entities' lifespans.
    pub(crate) entities: Entities, 
    /// Registered Data Types.
    pub(crate) components: Components, 
    pub(crate) bundles: Bundles,
    /// Registered sets of `Components`.
    // pub(crate) archetypes: Archetypes, 
    /// Handles all the data storage of the `World`.
    pub(crate) storages: Storages, 
}

impl Default for World {
    fn default() -> Self {
        Self {
            entities: Entities::default(),
            components: Components::default(),
            bundles: Bundles::default(),
            storages: Storages::default(),
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    // region: Component & Resource registration
    /// Registers the specified [`Component`] into the `World`, assigning it an unique [`ComponentId`].
    pub fn register_component<T: Component>(&mut self) -> ComponentId {
        self.components.register_component::<T>()
    }

    /// Registers the specified [`Resource`] into the `World`, assigning it an unique [`ComponentId`].
    ///
    /// Registering a [`Resource`] does not insert any value into the `World`.
    /// For that you can use either [`World::init_resource`] or [`World::insert_resource`].
    pub fn register_resource<R: Resource>(&mut self) -> ComponentId {
        self.components.register_resource::<R>()
    }

    /// Initializes a [`Resource`], assigning it an unique [`ComponentId`].
    ///
    /// The value returned by [`Default::default`] will be used.
    /// If the resource already had a value registered, nothing happens.
    pub fn init_resource<R: Resource + Default>(&mut self) {
        let id = self.components.register_resource::<R>();
        let store = self
            .storages
            .resources
            .fetch_resource_storage(id, &mut self.components);

        if !store.has_value() {
            let value = R::default();
            OwnPtr::from(value, |ptr| unsafe {
                // SAFETY: ComponentId was just registered for the type `R`
                store.insert_data(ptr);
            });
        }
    }

    /// Inserts a [`Resource`] into the `World` with the given `value`.
    ///
    /// This will replace any existing value.
    pub fn insert_resource<R: Resource>(&mut self, value: R) {
        let id = self.components.register_resource::<R>();
        OwnPtr::from(value, |ptr| unsafe {
            // SAFETY: ComponentId was just registered for the type of `value`
            self.insert_resource_by_id(id, ptr);
        });
    }

    /// Inserts a `Resource` of type `R` inside the `World`, replacing any old value which may be present.
    ///
    /// Safety
    /// The caller must ensure that the value pointed to by `value` corresponds to that of `component_id`.
    #[inline]
    pub(crate) unsafe fn insert_resource_by_id(&mut self, id: ComponentId, ptr: OwnPtr<'_>) {
        let store = self
            .storages
            .resources
            .fetch_resource_storage(id, &mut self.components);

        // SAFETY: the caller ensures that `value` and `id` refers to the same value.
        unsafe { store.insert_data(ptr) };
    }

    // endregion

    // Getters for Resources
}
