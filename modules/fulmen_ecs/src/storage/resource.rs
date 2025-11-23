// --- Imports
use crate::component::{ComponentId, Components};
use crate::utils::SparseSet;

use blob::Blob;

#[derive(Default)]
pub struct ResourceStorage {
    pub(crate) resources: SparseSet<ComponentId, Blob>, // Set [ComponentId -> Data]
}

impl ResourceStorage {
    /// Tries to get a reference to the underlying storage for a given [`ComponentId`].
    ///
    /// # Safety
    /// The caller must ensure the following:
    ///  - `id` is registered [`ComponentId`] inside of `components` as a [`Resource`](`crate::Resource`).
    pub unsafe fn get_resource_storage(&self, component_id: ComponentId) -> Option<&Blob> {
        self.resources.get(component_id)
    }

    /// Tries to get a mutable reference to the underlying storage for a given [`ComponentId`].
    ///
    /// # Safety
    /// The caller must ensure the following:
    ///  - `id` is registered [`ComponentId`] inside of `components` as a [`Resource`](`crate::Resource`).
    pub unsafe fn get_resource_storage_mut(
        &mut self,
        component_id: ComponentId,
    ) -> Option<&mut Blob> {
        self.resources.get_mut(component_id)
    }

    /// Fetches the underlying storage for a given [`ComponentId`] based on it's registered
    /// [`ComponentDef`](crate::component::ComponentDef).
    ///
    /// If there was no resource storage registered for that [`ComponentId`], it will be initialized.
    ///
    /// # Safety
    /// The caller must ensure the following:
    ///  - `id` is registered [`ComponentId`] inside of `components` as a [`Resource`](`crate::Resource`).
    pub unsafe fn fetch_resource_storage(
        &mut self,
        component_id: ComponentId,
        components: &Components,
    ) -> &mut Blob {
        // Look up if the resource is listed on the storage, or create it if it wasn't.
        self.resources.get_or_insert_with(component_id, || unsafe {
            // Look up if the component has been registered
            let info = components
                .get_info(component_id)
                .expect("Component has not been registered");

            // SAFETY: `ComponentInfo` is considered to be valid, thus `layout` and `drop_fn` are valid.
            Blob::with_layout_unchecked(info.layout(), info.drop_fn())
        })
    }
}
