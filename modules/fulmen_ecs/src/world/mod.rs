// --- Modules
#[cfg(test)]
mod tests;

// --- Imports
use crate::bundle::Bundles;
use crate::component::{Component, ComponentId, Components};
use crate::entity::Entities;
use crate::resource::Resource;
use crate::storage::Storages;

use fulmen_ptr::OwnPtr;

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
    pub fn register_component<C: Component>(&mut self) -> ComponentId {
        self.components.register_component::<C>()
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

    // region: Getters for ComponentIDs

    /// Gets the [`ComponentId`] of the given [`Component`] `C` on this `World` if it exists.
    pub fn get_component_id<C: Component>(&self) -> Option<ComponentId> {
        self.components.component_id::<C>()
    }

    /// Gets the [`ComponentId`] of the given [`Resource`] `R` on this `World` if it exists.
    pub fn get_resource_id<R: Resource>(&self) -> Option<ComponentId> {
        self.components.resource_id::<R>()
    }

    // endregion

    // region: Getters for Resources

    /// Gets a reference to a [`Resource`] of the given type if it exists.
    ///
    /// Will return `None` if the resource `R` was not registered  o it wasn't initialized.
    ///
    /// *See also: [`World::register_resource`]*
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        // Check for component registration
        if let Some(id) = self.get_resource_id::<R>()
            && let Some(storage) = unsafe { self.storages.resources.get_resource_storage(id) }
        {
            if storage.has_value() {
                // SAFETY: `storage` was created for type `R`, and initialization was checked
                unsafe { Some(storage.downcast_ref_unchecked::<R>()) }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets a reference to a [`Resource`] of the given type if it exists.
    ///
    /// Will return `None` if the resource `R` was not registered or it wasn't initialized.
    ///
    /// *See also: [`World::register_resource`]*
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        // Check for component registration
        if let Some(id) = self.get_resource_id::<R>()
            && let Some(storage) = unsafe { self.storages.resources.get_resource_storage_mut(id) }
        {
            if storage.has_value() {
                // SAFETY: `storage` was created for type `R`, and initialization was checked
                unsafe { Some(storage.downcast_mut_unchecked::<R>()) }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets a reference to the [`Resource`] of the specified type.
    ///
    /// # Panics
    /// Will panic if the resource was not initialized.
    pub fn resource<R: Resource>(&self) -> &R {
        match self.get_resource::<R>() {
            Some(res) => res,
            None => panic!(
                "Tried to acquire uninitialized resource `{}` from `World`",
                core::any::type_name::<R>()
            ),
        }
    }

    /// Gets a mutable reference to the [`Resource`] of the specified type.
    ///
    /// # Panics
    /// Will panic if the resource was not initialized.
    pub fn resource_mut<R: Resource>(&mut self) -> &mut R {
        match self.get_resource_mut::<R>() {
            Some(res) => res,
            None => panic!(
                "Tried to mutably acquire uninitialized resource `{}` from `World`",
                core::any::type_name::<R>()
            ),
        }
    }

    // TODO: Non Send resources

    // endregion
}
