use crate::{
    component::{ComponentId, Components},
    utils::SparseSetIndex,
};

use core::any::TypeId;

/// A value used to uniquelly identify the type of a [`Bundle`].
///
/// `BundleId` is used instead of [`TypeId`] to ensure they are incremental in nature.
///
/// ## SAFETY
/// * This value is only guaranteed to be unique inside the same [`World`].
///
/// [`World`]: crate::world::World
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct BundleId(pub usize);

impl BundleId {
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

/// Handles all the info about the [`Component Bundles`](`Bundle`) that have been registered in a [`World`].
///
/// [`World`]: crate::world::World
#[derive(Debug, Default)]
pub struct Bundles {
    // bundle_infos: Vec<BundleInfo>,
}
