// --- Imports
use crate::component::{ComponentId, Components};
use crate::utils::SparseSet;

use blob::Blob;

#[derive(Default)]
pub struct ResourceStorage {
    pub(crate) resources: SparseSet<ComponentId, Blob>, // Set [ComponentId -> Data]
}

impl ResourceStorage {
    /// Fetches the underlying storage for a given [`ComponentId`] based on it's registered [`ComponentDef`].
    ///
    /// If there was no storage registered for that [`ComponentId`].
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
