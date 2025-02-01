use crate::component::{Component, ComponentId, Components};
use crate::resource::Resource;
use crate::storage::Storage;

pub struct World {
    // pub(crate) entities: Entities, // Set of EntityIDs
    pub(crate) components: Components, // Registered Data Types
    pub(crate) storage: Storage,
}

impl Default for World {
    fn default() -> Self {
        Self {
            components: Components::default(),
            storage: Storage::default(),
        }
    }
}

impl World {
    // pub fn resource_id<T: Resource>(&self) -> Option<usize> {
    //     None
    // }

    ///
    pub fn register_component<T: Component>(&mut self) -> ComponentId {
        self.components.register_component::<T>(&mut self.storage)
    }

    ///
    pub fn register_resource<R: Resource>(&mut self) -> ComponentId {
        self.components.register_resource::<R>()
    }

    ///
    pub fn init_resource<R: Resource + Default>(&mut self) {
        let id = self.components.register_resource::<R>();

        todo!("if resource not found, insert default value")
    }

    ///
    pub fn store_resource<R: Resource>(&mut self, value: R) {
        let id = self.components.register_resource::<R>();
        // SAFETY: ComponentId was just registered
        unsafe { self.insert_resource_by_id(id, value); }
    }

    unsafe fn insert_resource_by_id<R: Resource>(&mut self, component_id: ComponentId, value: R)  {
        todo!();
    }

    // Getters for Resources

    // pub fn store_resource()
}
