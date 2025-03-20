use crate::resource::Resource;
use crate::storage::Storages;
use crate::utils::TypeIdMap;
use crate::world::World;

use fulmen_ptr::DropFn;

use std::alloc;
use std::any::TypeId;
use std::{mem, ptr::NonNull};

/// A value used to uniquelly identify the type of a [`Component`]/[`Resource`](crate::Resource).
///
/// A new `ComponentId` will be created for each `Component` or `Resource` type registered into a [`World`].
/// This is done via the [`World::store_resource`](crate::World::store_resource)
/// or [`World::register_component`](crate::World::register_component) methods.
///
/// `ComponentId` is used instead of [`TypeId`] to ensure that `ComponentId`s are incremental in natrure.
///
/// ## SAFETY
/// * This value is only guaranteed to be unique inside the same [`World`],
///   and thus using a `ComponentId` outside of it's respective [`World`] is undefined behaviour.
///
/// * Having more than [`usize::MAX`] different registered [`Component`]s will result in the program crashing,
///   as that is the upper component limit.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId(pub usize);

impl ComponentId {
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.0
    }
}

pub trait Component: Send + Sync + 'static {
    /// A constant indicating the storage type used for this component.
    const STORAGE_TYPE: StorageType;

    // /// Called when registering this component, allowing mutable access to its [`ComponentHooks`].
    // fn register_component_hooks(_hooks: &mut ComponentHooks) {}

    // /// Registers required components.
    // fn register_required_components(
    //     _component_id: ComponentId,
    //     _components: &mut Components,
    //     _storages: &mut Storages,
    //     _required_components: &mut RequiredComponents,
    //     _inheritance_depth: u16,
    // ) {}
}

#[macro_export]
macro_rules! impl_component {
    ( $t:ty ) => {
        impl_component!($t, fulmen_ecs::component::StorageType::SparseSet);
    };
    ( $t:ty, $storage_type:ident ) => {
        impl fulmen_ecs::component::Component for $t {
            const STORAGE_TYPE: fulmen_ecs::component::StorageType =
                fulmen_ecs::component::StorageType::$storage_type;
        }
    };
    ( $t:ty, $storage_type:expr ) => {
        impl fulmen_ecs::component::Component for $t {
            const STORAGE_TYPE: fulmen_ecs::component::StorageType = $storage_type;
        }
    };
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum StorageType {
    #[default]
    SparseSet, // Faster addition and removal
    Table, // Faster iteration
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    definition: ComponentDef,
}

impl ComponentInfo {
    pub(crate) fn new(id: ComponentId, definition: ComponentDef) -> Self {
        Self { id, definition }
    }

    #[inline]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.definition.type_id
    }

    #[inline]
    pub fn storage_type(&self) -> StorageType {
        self.definition.storage_type
    }

    #[inline]
    pub fn layout(&self) -> alloc::Layout {
        self.definition.layout
    }

    #[inline]
    pub fn drop_fn(&self) -> Option<DropFn> {
        self.definition.drop_fn
    }
}

#[derive(Debug, Clone)]
pub struct ComponentDef {
    type_id: TypeId,
    storage_type: StorageType,
    layout: alloc::Layout,
    drop_fn: Option<DropFn>,
}

impl ComponentDef {
    pub fn new<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            storage_type: T::STORAGE_TYPE,
            layout: alloc::Layout::new::<T>(),
            drop_fn: fulmen_ptr::get_drop_fn::<T>(),
        }
    }

    pub fn new_resource<R: Resource>() -> Self {
        Self {
            type_id: TypeId::of::<R>(),
            storage_type: StorageType::Table,
            layout: alloc::Layout::new::<R>(),
            drop_fn: fulmen_ptr::get_drop_fn::<R>(),
        }
    }
}

/// Handles all the info about the [`Component`]s of a [`World`]
#[derive(Debug, Default)]
pub struct Components {
    component_infos: Vec<ComponentInfo>,
    comp_indeces: TypeIdMap<ComponentId>,
    res_indeces: TypeIdMap<ComponentId>,
}

impl Components {
    #[inline]
    pub fn register_component<T: Component>(&mut self, storage: &mut Storages) -> ComponentId {
        // Get the unique id for the Type
        let type_id = TypeId::of::<T>();

        // Look up if the component has already been registered, or register it if it wasn't.
        let comp_id = {
            let Components {
                component_infos,
                comp_indeces,
                ..
            } = self;

            *comp_indeces.entry(type_id).or_insert_with(|| {
                let id = ComponentId::new(component_infos.len());
                let info = ComponentInfo::new(id, ComponentDef::new::<T>());

                // if info.definition.storage_type == StorageType::SparseSet { }
                // TODO: Initialize a SparseSet for the Type if necessary in the data store
                component_infos.push(info);

                // TODO; Handle recursive required components (aka parenting data)

                id
            })
        };

        comp_id
    }

    pub fn register_resource<R: Resource>(&mut self) -> ComponentId {
        // Get the unique id for the Type
        let type_id = TypeId::of::<R>();

        // Look up if the component has already been registered, or register it if it wasn't.
        let comp_id = {
            let Components {
                component_infos,
                res_indeces,
                ..
            } = self;

            *res_indeces.entry(type_id).or_insert_with(|| {
                let id = ComponentId::new(component_infos.len());
                let info = ComponentInfo::new(id, ComponentDef::new_resource::<R>());

                component_infos.push(info);
                id
            })
        };

        comp_id
    }

    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.component_infos.get(id.index())
    }

    /// Equivalent of [`Components::component_id()`].
    #[inline]
    pub fn get_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.comp_indeces.get(&type_id).copied()
    }

    /// Equivalent of [`Components::resource_id()`].
    #[inline]
    pub fn get_resource_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.res_indeces.get(&type_id).copied()
    }

    #[inline]
    pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.get_id(TypeId::of::<T>())
    }

    #[inline]
    pub fn resource_id<R: Resource>(&self) -> Option<ComponentId> {
        self.get_resource_id(TypeId::of::<R>())
    }
}
