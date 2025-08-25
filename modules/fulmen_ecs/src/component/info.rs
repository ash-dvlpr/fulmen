// --- Imports
use super::{Component, StorageType};

use crate::resource::Resource;
use crate::utils::{SparseSetIndex, TypeIdMap};

use fulmen_ptr::DropFn;

use std::alloc;
use std::any::TypeId;

/// A value used to uniquelly identify the type of a [`Component`]/[`Resource`](crate::Resource).
///
/// A new `ComponentId` will be created for each `Component` or `Resource` type registered into a [`World`](crate::World).
/// This will be usually done via the [`World::register_component`](crate::World::register_component)
/// or [`World::init_resource`](crate::World::init_resource) methods.
///
/// `ComponentId` is used instead of [`TypeId`] to ensure they are incremental in nature.
///
/// ## SAFETY
/// * This value is only guaranteed to be unique inside the same [`World`](crate::World),
///   and thus using a `ComponentId` outside of it's respective [`World`](crate::World) is undefined behaviour.
///
/// * Having more than [`usize::MAX`] different registered [`Component`]s will result in the program crashing,
///   as that is the upper component limit.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
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

impl SparseSetIndex for ComponentId {
    #[inline]
    fn sparse_set_index(&self) -> usize {
        self.index()
    }

    #[inline]
    fn get_sparse_set_index(value: usize) -> Self {
        Self(value)
    }
}

/// Struct that hold general metadata about registered [`Components`](`Component`) types
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

/// Handles all the metadata about the [`Components`](`Component`) of a [`World`]
#[derive(Debug, Default)]
pub struct Components {
    component_infos: Vec<ComponentInfo>,
    comp_indices: TypeIdMap<ComponentId>,
    res_indices: TypeIdMap<ComponentId>,
}

impl Components {
    /// The total number of [`Components`](`Component`) registered in the [`World`].
    #[inline]
    pub fn len(&self) -> usize {
        self.component_infos.len()
    }

    /// Returns true if no [`Components`](`Component`) registered in the [`World`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the [`ComponentInfo`] of an specific component.
    ///
    /// Returns `None` if there's no [`Component`] registered in this [`World`]
    /// with that `ComponentId`.
    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.component_infos.get(id.index())
    }

    /// Gets the [`ComponentId`] corresponding to a component type.
    ///
    /// Returns `None` if the `type_id` is not registered as a [`Component`] in the [`World`].
    #[inline]
    pub fn get_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.comp_indices.get(&type_id).copied()
    }

    /// Gets the [`ComponentId`] corresponding to a resource type.
    ///
    /// Returns `None` if the `type_id` is not registered as a [`Resource`] in the [`World`].
    #[inline]
    pub fn get_resource_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.res_indices.get(&type_id).copied()
    }

    /// Generic shorthand for [`Components::get_id`]
    #[inline]
    pub fn component_id<T: Component>(&self) -> Option<ComponentId> {
        self.get_id(TypeId::of::<T>())
    }

    /// Generic shorthand for [`Components::get_resource_id`]
    #[inline]
    pub fn resource_id<R: Resource>(&self) -> Option<ComponentId> {
        self.get_resource_id(TypeId::of::<R>())
    }

    /// Registers the [`ComponentInfo`] for a new [`Component`] if it wasn't already registered.
    #[inline]
    pub(crate) fn register_component<T: Component>(&mut self) -> ComponentId {
        // Get the unique id for the Type
        let type_id = TypeId::of::<T>();

        // Look up if the component has already been registered, or register it if it wasn't.
        let comp_id = {
            let Self {
                component_infos,
                comp_indices,
                ..
            } = self;

            *comp_indices.entry(type_id).or_insert_with(|| {
                let id = ComponentId::new(component_infos.len());
                let info = ComponentInfo::new(id, ComponentDef::new::<T>());

                component_infos.push(info);

                // TODO; Handle recursive required components (aka parenting data)

                id
            })
        };

        comp_id
    }

    /// Registers the [`ComponentInfo`] for a new [`Resource`] if it wasn't already registered.
    pub(crate) fn register_resource<R: Resource>(&mut self) -> ComponentId {
        // Get the unique id for the Type
        let type_id = TypeId::of::<R>();

        // Look up if the component has already been registered, or register it if it wasn't.
        let comp_id = {
            let Components {
                component_infos,
                res_indices,
                ..
            } = self;

            *res_indices.entry(type_id).or_insert_with(|| {
                let id = ComponentId::new(component_infos.len());
                let info = ComponentInfo::new(id, ComponentDef::new_resource::<R>());

                component_infos.push(info);
                id
            })
        };

        comp_id
    }
}
