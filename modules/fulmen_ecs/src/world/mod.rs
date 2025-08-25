use fulmen_ptr::OwnPtr;

use crate::bundle::Bundles;
use crate::component::{Component, ComponentId, Components};
use crate::entity::Entities;
use crate::resource::Resource;
use crate::storage::Storages;

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
        let value = R::default();
        let id = self.components.register_resource::<R>();
        OwnPtr::from(value, |ptr| unsafe {
            // SAFETY: `id` was just registered for the type `R`.
            self.insert_resource_by_id::<false>(id, ptr);
        });
    }

    /// Inserts a [`Resource`] into the `World` with the given `value`.
    ///
    /// This will replace any existing value.
    pub fn insert_resource<R: Resource>(&mut self, value: R) {
        let id = self.components.register_resource::<R>();
        OwnPtr::from(value, |ptr| unsafe {
            // SAFETY: `id` was just registered for the type `R`.
            self.insert_resource_by_id::<true>(id, ptr);
        });
    }

    /// Inserts a [`Resource`] inside the `World`.
    ///
    /// If `OVERRIDE` is `true`, it will replace any value which may already be present.
    ///
    /// # Safety
    /// The caller must ensure the following:
    ///  - `id` comes from [`Resource`] type a registered on this `World`.
    ///  - The type of the value pointed to by `ptr` corresponds to that of `id`.
    #[inline(always)]
    pub(crate) unsafe fn insert_resource_by_id<const OVERRIDE: bool>(
        &mut self,
        id: ComponentId,
        ptr: OwnPtr<'_>,
    ) {
        // SAFETY: the caller ensures that `id` is that of a registered resource.
        let store = unsafe {
            self.storages
                .resources
                .fetch_resource_storage(id, &mut self.components)
        };

        if OVERRIDE || !store.has_value() {
            // SAFETY: the caller ensures that `ptr` and `id` refers to the same type.
            unsafe { store.insert_data(ptr) };
        }
    }

    // endregion

    // region: Getters for Resources

    // TODO:

    // endregion
}
