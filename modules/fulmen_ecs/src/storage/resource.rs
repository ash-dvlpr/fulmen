// --- Imports
use crate::component::{ComponentId, Components};
use crate::utils::TypeIdMap;

use blob::Blob;

#[derive(Default)]
pub struct ResourceStorage {
    // TODO: refactor into using SparseSet, using ComponentId as a key
    pub(crate) resources: TypeIdMap<Blob>, // Set [ComponentId -> Data]
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
        // Look up if the component has been registered
        let info = components
            .get_info(component_id)
            .expect("Component has not been registered");

        // Look up if the resource is listed on the storage, or create it if it wasn't.
        self.resources
            .entry(info.type_id())
            .or_insert_with(|| unsafe {
                // SAFETY: `ComponentInfo` is considered to be valid, thus `layout` and `drop_fn` are valid.
                Blob::with_layout_unchecked(info.layout(), info.drop_fn())
            })
    }
}
