pub mod blob;
pub mod sparse;

use blob::BlobData;

use crate::component::{ComponentId, ComponentDef, Components, Component};
use crate::utils::TypeIdMap;

/// The backing data of a [`World`](crate::world::World)
#[derive(Default)]
pub struct Storage {
    pub(crate) resources: TypeIdMap<BlobData>, // Set [ComponentId -> Data]
}

impl Storage {
    /// Fetches the underlying storage for a given [`Component`] based on it's registered [`ComponentDef`].
    pub fn fetch_resource_storage(
        &mut self,
        component_id: ComponentId,
        components: &Components,
    ) -> &mut BlobData {
        // Look up if the component has been registered
        let info = components
            .get_info(component_id)
            .expect("Component has not been registered");

        // Look up if the resource is listed on the storage, or initialize it if it wasn't.
        self.resources
            .entry(info.type_id())
            .or_insert_with(|| unsafe { BlobData::new(info.layout()) })
    }
}
