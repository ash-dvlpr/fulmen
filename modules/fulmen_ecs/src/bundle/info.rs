// --- Imports
use crate::{
    bundle::Bundle,
    component::{ComponentId, Components},
    utils::{SparseSetIndex, TypeIdMap},
};

use core::any::TypeId;
use std::collections::HashSet;

/// A value used to uniquelly identify the type of a [`Bundle`].
///
/// `BundleId` is used instead of [`TypeId`] to ensure they are incremental in nature.
///
/// # Safety
/// - This value is only guaranteed to be unique inside the same [`World`].
///
/// [`World`]: crate::world::World
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct BundleId(pub usize);

impl BundleId {
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.0
    }
}

impl SparseSetIndex for BundleId {
    #[inline]
    fn sparse_set_index(&self) -> usize {
        self.index()
    }

    #[inline]
    fn get_sparse_set_index(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct BundleInfo {
    id: BundleId,
    component_ids: Box<[ComponentId]>,
}

impl BundleInfo {
    /// Create a new [`BundleInfo`].
    ///
    /// # Safety
    /// All the `ComponentIDs` in `component_ids` must be valid for the owning `World`,
    /// and must be in the same order as specified by the `Bundle` trait.
    ///
    /// # Panics
    /// Will panic if the `component_ids` contains duplicated elements.
    unsafe fn new(
        bundle_type_name: &'static str,
        id: BundleId,
        component_ids: Vec<ComponentId>,
    ) -> Self {
        // Check for duplicated component ids
        {
            let mut seen = HashSet::new();
            for &id in &component_ids {
                if !seen.insert(id) {
                    panic!(
                        "Bundle Type '{}' contains duplicated Components",
                        bundle_type_name
                    );
                }
            }
        }

        Self {
            id,
            component_ids: component_ids.into_boxed_slice(),
        }
    }

    #[inline]
    pub fn id(&self) -> BundleId {
        self.id
    }

    #[inline]
    pub fn component_ids(&self) -> &[ComponentId] {
        self.component_ids.as_ref()
    }
}

/// Handles all the info about the [`Component Bundles`](`Bundle`) that have been registered in a [`World`].
///
/// [`World`]: crate::world::World
#[derive(Debug, Default)]
pub struct Bundles {
    bundle_infos: Vec<BundleInfo>,
    bundle_indices: TypeIdMap<BundleId>,
}

impl Bundles {
    /// The total number of [`Bundles`](`Bundle`) registered in the [`World`].
    #[inline]
    pub fn len(&self) -> usize {
        self.bundle_infos.len()
    }
    /// Returns true if no [`Bundles`](`Bundle`) registered in the [`World`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the [`BundleInfo`] of an specific component bundle.
    ///
    /// Returns `None` if there's no [`Bundle`] registered in this [`World`] with that `BundleId`.
    #[inline]
    pub fn get(&self, bundle_id: BundleId) -> Option<&BundleInfo> {
        self.bundle_infos.get(bundle_id.index())
    }

    /// Gets the [`BundleId`] corresponding to a type.
    ///
    /// Returns `None` if the `type_id` is not registered as a [`Bundle`] in the [`World`].
    #[inline]
    pub fn get_id(&self, type_id: TypeId) -> Option<BundleId> {
        self.bundle_indices.get(&type_id).cloned()
    }

    /// Registers the [`BundleInfo`] for a new [`Component Bundle`](`Bundle`) if it doesn't exist.
    ///
    /// # Panics
    /// Will panic if the [`Bundle`] contains duplicated [`Components`](`Component`).
    #[inline]
    pub(crate) fn register_bundle<B: Bundle>(
        &mut self,
        world_components: &mut Components,
    ) -> BundleId {
        // Get the unique id for the Type
        let type_id = TypeId::of::<B>();

        // Look up if the component has already been registered, or register it if it wasn't.
        let bundle_id = {
            let Self {
                bundle_infos,
                bundle_indices,
                ..
            } = self;

            *bundle_indices.entry(type_id).or_insert_with(|| {
                // Collect the `ComponentId`s for all the components on the Bundle.
                let mut component_ids = Vec::new();
                B::get_or_register_component_ids(world_components, &mut |id| {
                    component_ids.push(id)
                });

                // Create the new ID
                let id = BundleId::new(bundle_infos.len());
                let type_name = core::any::type_name::<B>();
                // SAFETY: `Bundle:: get_or_register_component_ids` ensures:
                // - The ids are valid for the containing `World`.
                // - The ids are in the order declared by the `Bundle`.
                // - It's expected for this method to panic if there are duplicated component in the bundle.
                let info = unsafe { BundleInfo::new(type_name, id, component_ids) };
                bundle_infos.push(info);

                // TODO; Handle recursive required components (aka parenting data)

                id
            })
        };

        bundle_id
    }
}
