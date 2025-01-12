use crate::{Error, Result};
use core::{marker::PhantomData, ptr::NonNull};
use std::{
    alloc::{self, Layout},
    num::NonZeroUsize,
};

/// Internal type erased BLOB used by the [`crate::Blob`] and [`crate::VecBlob`] implementations.
///
/// It handles allocation and resizing, while delegating size and capacity to the users for memory performance reasons.
/// Will lazily allocate if created with a `capacity` of 0.
///
/// Because of type erasure, users must also provide the `drop_fn` for the stored values when necessary.
pub(super) struct UnsafeBlob {
    data: Option<NonNull<u8>>,
    pub drop_fn: Option<unsafe fn(*mut u8)>,
    item_layout: Layout,
    _marker: PhantomData<u8>,
}

impl UnsafeBlob {
    /// Creates a new `UnsafeBlob` with the specified `capacity`.
    /// Will lazily allocate if `capacity` is 0.
    ///
    /// Internal buffer size is calculated based on the `item_layout`.
    ///
    /// # Returns
    /// - `Err` if `item_layout.size() == 0`
    ///
    /// # Safetly
    /// `drop_fn` should be safe with any value stored inside the Blob, as long as the `drop_fn` corresponds to the Type of the values being stored,
    /// If the `drop_fn` is `None`, values will be leaked. This should be set to none based on [`core::mem::needs_drop`].
    #[inline]
    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(*mut u8)>,
        capacity: usize,
    ) -> Result<UnsafeBlob> {
        if item_layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        // SAFETY: Just checked for invalid layouts
        Ok(unsafe { Self::with_capacity_unchecked(item_layout, drop_fn, capacity) })
    }

    /// Creates a new `UnsafeBlob` with the specified `capacity`.
    /// Will lazily allocate if `capacity` is 0.
    ///
    /// Internal buffer size is calculated based on the `item_layout`.
    ///
    /// # Returns
    /// - `Err` if `item_layout.size() == 0`
    ///
    /// # Safetly
    /// `drop_fn` should be safe with any value stored inside the Blob, as long as the `drop_fn` corresponds to the Type of the values being stored,
    /// If the `drop_fn` is `None`, values will be leaked. This should be set to none based on [`core::mem::needs_drop`].
    ///
    /// The caller must ensure that the `item_layout` has a `size > 0` and has a propper alignement.
    /// For for a safer variant, see [`UnsafeBlob::with_capacity`].
    #[inline]
    pub unsafe fn with_capacity_unchecked(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(*mut u8)>,
        capacity: usize,
    ) -> UnsafeBlob {
        debug_assert!(item_layout.size() > 0, "Size must be non zero");

        let mut blob = UnsafeBlob {
            data: None,
            drop_fn,
            item_layout,
            _marker: PhantomData,
        };

        if capacity > 0 {
            // SAFETY: we just checked that capacity is non zero
            blob.alloc_buffer(NonZeroUsize::new_unchecked(capacity));
        }

        blob
    }

    /// The [`core::alloc::Layout`] of the values stored inside the `UnsafeBlob`.
    #[inline]
    pub fn item_layout(&self) -> Layout {
        self.item_layout
    }

    /// Wether or not the `UnsafeBlob` has been allocated.
    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.data.is_some()
    }

    /// Allocate a buffer for the `UnsafeBlob`. Only use this when initializing the `UnsafeBlob`.
    /// If the `UnsafeBlob` has already been allocated, use [`Self::realloc_buffer`] instead.
    ///
    /// # Safety
    /// The `UnsafeBlob` must not have been allocated. This can be checked via [`Self::is_allocated`].
    ///
    /// See [`GlobalAlloc::alloc`].
    #[inline]
    pub(super) unsafe fn alloc_buffer(&mut self, capacity: NonZeroUsize) {
        debug_assert!(self.item_layout.size() > 0, "Size must be non zero");
        debug_assert!(
            !self.is_allocated(),
            "UnsafeBlob shouldn't be initalized more than once"
        );

        // SAFETY: item_layout has non-zero size and capacity > 0 so the array_layout will be valid.
        let arr_layout = array_layout(&self.item_layout, capacity.into()).unwrap();
        let ptr = unsafe { std::alloc::alloc(arr_layout) };
        self.data = NonNull::new(ptr)
            .unwrap_or_else(|| std::alloc::handle_alloc_error(arr_layout))
            .into();
    }

    pub(super) unsafe fn realloc_buffer(&mut self, capacity: usize, new_capacity: usize) {
        todo!();
    }

    pub(super) unsafe fn drop_buffer(&mut self, size: usize, capacity: usize) {
        todo!();
    }
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
pub(super) fn array_layout(item_layout: &Layout, capacity: usize) -> Option<Layout> {
    let (array_layout, offset) = layout_repeat(item_layout, capacity)?;
    debug_assert!(
        item_layout.size() == offset,
        "Offset of an array_layout for an item_layout should match the item_layout's size."
    );

    Some(array_layout)
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
#[inline]
fn layout_repeat(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded = layout.pad_to_align();
    let alloc_size = padded.size().checked_mul(n)?;

    // SAFETY: layout.align() is already known to be valid and
    // alloc_size has been padded.
    Some((
        unsafe { Layout::from_size_align_unchecked(alloc_size, layout.align()) },
        padded.size(),
    ))
}

#[cfg(test)]
mod tests {
    use super::layout_repeat;
    use std::alloc::Layout;

    #[allow(unused)]
    struct TestStruct(usize, u8, u8);

    #[test]
    fn test_array_layout() {
        let layout = Layout::new::<TestStruct>();

        let (array_layout, offset) = layout_repeat(&layout, 4).unwrap();
        debug_assert_eq!(layout.size(), offset);
        debug_assert_eq!(array_layout.size() / 4, layout.size());
    }
}
