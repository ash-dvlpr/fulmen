use crate::{Error, Result};
use fulmen_ptr::{util::array_layout, *};

use core::cell::UnsafeCell;
use core::num::NonZeroUsize;
use core::{alloc::Layout, marker::PhantomData, ptr::NonNull};

/// Internal type erased BLOB used by the [`crate::Blob`] and [`crate::VecBlob`] implementations.
///
/// It handles allocation and resizing, while delegating size and capacity to the users for memory performance reasons.
/// Will lazily allocate if created with a `capacity` of 0.
///
/// Because of type erasure, users must also provide the `drop_fn` for the stored values when necessary.
pub(super) struct UnsafeBlob {
    data: Option<NonNull<u8>>,
    pub drop_fn: Option<unsafe fn(OwnPtr)>,
    item_layout: Layout,
    _marker: PhantomData<u8>,
}

impl UnsafeBlob {
    /// Creates a new [`UnsafeBlob`] with the specified `capacity`.
    /// Will lazily allocate if `capacity` is 0.
    ///
    /// Internal buffer size is calculated based on the `item_layout`.
    ///
    /// # Returns
    /// - `Err` if `item_layout.size() == 0`
    ///
    /// # Safetly
    /// `drop_fn` should be safe with any value stored inside the Blob,
    /// as long as the `drop_fn` corresponds to the Type of the values being stored.
    /// If the `drop_fn` is `None`, values will be leaked.
    /// This should be set to none based on [`core::mem::needs_drop`].
    #[inline]
    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwnPtr)>,
        capacity: usize,
    ) -> Result<UnsafeBlob> {
        if item_layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        // SAFETY: Just checked for invalid layouts
        Ok(unsafe { Self::with_capacity_unchecked(item_layout, drop_fn, capacity) })
    }

    /// Creates a new [`UnsafeBlob`] with the specified `capacity`.
    /// Will lazily allocate if `capacity` is 0.
    ///
    /// Internal buffer size is calculated based on the `item_layout`.
    ///
    /// # Returns
    /// - `Err` if `item_layout.size() == 0`
    ///
    /// # Safetly
    /// `drop_fn` should be safe with any value stored inside the Blob,
    /// as long as the `drop_fn` corresponds to the Type of the values being stored,
    /// If the `drop_fn` is `None`, values will be leaked.
    /// This should be set to none based on [`core::mem::needs_drop`].
    ///
    /// The caller must ensure that the `item_layout` has a `size > 0` and has a propper alignement.
    /// For for a safer variant, see [`UnsafeBlob::with_capacity`].
    #[inline]
    pub unsafe fn with_capacity_unchecked(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwnPtr)>,
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

    /// The [`core::alloc::Layout`] of the values stored inside the [`UnsafeBlob`].
    #[inline]
    pub fn item_layout(&self) -> Layout {
        self.item_layout
    }

    /// Wether or not the [`UnsafeBlob`] has been allocated.
    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.data.is_some()
    }

    /// Wether or not the [`UnsafeBlob`] has a drop_fn registered.
    #[inline]
    pub fn has_drop(&self) -> bool {
        self.drop_fn.is_some()
    }

    /// Gets the [`Ptr`] to the start of the underlying buffer.
    #[inline]
    pub unsafe fn get_ptr(&self) -> Option<Ptr<'_>> {
        // SAFETY: data is valid for as long as 'self.
        self.data.and_then(|ptr| Some(unsafe { Ptr::new(ptr) }))
    }

    /// Gets the [`PtrMut`] to the start of the underlying buffer.
    #[inline]
    pub unsafe fn get_ptr_mut(&self) -> Option<PtrMut<'_>> {
        // SAFETY: data is valid for as long as 'self.
        self.data.and_then(|ptr| Some(unsafe { PtrMut::new(ptr) }))
    }

    /// Borrows the element at `index`, with no bounds checking.
    ///
    /// # Safety
    /// - `index` < the true current `len` of the [`UnsafeBlob`].
    ///
    /// *`len` refers to the length of the array, the number of elements
    /// that have been initialized, and thus are safe to read.*
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> Ptr<'_> {
        debug_assert!(
            self.is_allocated(),
            "UnsafeBlob should be initalized before attempting to access it's buffer"
        );

        let elem_size = self.item_layout.size();
        // SAFETY:
        // - The caller ensures `index` fits inside the buffer's allocation and the blob is allocated.
        // - `elem_size` is the size of the stored element's errased type, so multiplying preserves alignement.
        unsafe { self.get_ptr().unwrap().byte_add(index * elem_size) }
    }

    /// Mutably borrows the element at `index`, with no bounds checking.
    ///
    /// # Safety
    /// - `index` < the true current `len` of the [`UnsafeBlob`].
    ///
    /// *`len` refers to the length of the array, the number of elements
    /// that have been initialized, and thus are safe to read.*
    #[inline]
    pub unsafe fn get_unchecked_mut(&self, index: usize) -> PtrMut<'_> {
        debug_assert!(
            self.is_allocated(),
            "UnsafeBlob should be initalized before attempting to access it's buffer"
        );

        let elem_size = self.item_layout.size();
        // SAFETY:
        // - The caller ensures `index` fits inside the buffer's allocation and the blob is allocated.
        // - `elem_size` is the size of the stored element's errased type, so multiplying preserves alignement.
        unsafe { self.get_ptr_mut().unwrap().byte_add(index * elem_size) }
    }

    /// Get a slice of the [`UnsafeBlob`] `slice_len` elements long.
    /// To get a slice of the whole Blob's contents, put the `len` of the [`UnsafeBlob`] into the `slice_len`.
    ///
    /// # Safety
    /// - `slice_len` <= `len` of the [`UnsafeBlob`].
    ///
    /// *`len` refers to the length of the array, the number of elements
    /// that have been initialized, and thus are safe to read.*
    pub unsafe fn get_slice<T>(&mut self, slice_len: usize) -> &[UnsafeCell<T>] {
        if let Some(ptr) = &self.data {
            unsafe {
                core::slice::from_raw_parts::<UnsafeCell<T>>(ptr.as_ptr() as *const _, slice_len)
            }
        } else {
            &[]
        }
    }

    /// Allocate a buffer for the [`UnsafeBlob`]. Only use this when initializing the `UnsafeBlob`.
    /// If the [`UnsafeBlob`] has already been allocated, use [`Self::realloc_buffer`] instead.
    ///
    /// # Safety
    /// The [`UnsafeBlob`] must not have been allocated. This can be checked via [`Self::is_allocated`].
    ///
    /// The caller must also keep track of the `new_capacity` after calling the method.
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

    /// Reallocate the internal buffer buffer of the [`UnsafeBlob`].
    /// One may call this method when you've reached `capacity` and need to increase the allocated memory.
    ///
    /// # Safety
    /// The caller is responsible for ensuring that the `capacity` is the true current capacity of the [`UnsafeBlob`].
    ///
    /// The caller must keep track of the `new_capacity` after calling the method.
    pub(super) unsafe fn realloc_buffer(
        &mut self,
        capacity: NonZeroUsize,
        new_capacity: NonZeroUsize,
    ) {
        debug_assert!(
            self.is_allocated(),
            "UnsafeBlob should be initalized before attempting to reallocate it"
        );
        debug_assert!(
            new_capacity > capacity,
            "New capacity should be greater than the previous one when reallocating an UnsafeBlob"
        );

        // SAFETY: was allocated beforehand so the array_layouts will be valid.
        let arr_layout = array_layout(&self.item_layout, capacity.into()).unwrap();
        let new_arr_layout = array_layout(&self.item_layout, new_capacity.into()).unwrap();

        // SAFETY: `ptr` was allocated previouslly, so it's valid. This also implies that `item_layout` is non-zero.
        let new_ptr = unsafe {
            let ptr = self.data.unwrap().as_ptr();
            std::alloc::realloc(ptr, arr_layout, new_arr_layout.size())
        };

        self.data = NonNull::new(new_ptr)
            .unwrap_or_else(|| std::alloc::handle_alloc_error(arr_layout))
            .into();
    }

    /// Clears the internal buffer, dropping all the elements inside of it.
    ///
    /// The behaviour should be comparable to that of [`Vec::clear`].
    ///
    /// # Parameters
    /// - `len`: The true current length of the [`UnsafeBlob`], shouldn't exceed the capacity.
    ///
    /// # Safety
    /// The buffer must be allocated
    /// The `drop_fn` of the `UnsafeBlob` must be safe to call for all elements from `i` in `0..len`.
    ///
    /// If the [`UnsafeBlob`] was properly constructed, this should be true for all elements that were intialized,
    /// as long as the `len` of the [`UnsafeBlob`] has been properly tracked.
    pub(super) unsafe fn clear_buffer(&mut self, len: usize) {
        debug_assert!(
            self.is_allocated(),
            "UnsafeBlob should be initalized before attempting to clear it"
        );

        if let Some(drop_fn) = self.drop_fn {
            // Set `self.drop_fn` to `None` before dropping any values to avoid double dropping values in case of an unwind.
            self.drop_fn = None;
            let elem_size = self.item_layout.size();
            for i in 0..len {
                // SAFETY:
                // - 0 <= `i` < `len`, so `i * elem_size` will be inside the buffer's allocation.
                // - `elem_size` is the size of the stored element's errased type, so multiplying preserves alignement.
                // - its's safe to `promote()` the pointer, as it will be left unreachable.
                let element_ptr = unsafe {
                    self.get_ptr_mut()
                        .unwrap()
                        .byte_add(i * elem_size)
                        .promote()
                };
                // SAFETY: `element` was stored inside the `UnsafeBlob`, so it's type must match that of `drop_fn`.
                unsafe { drop_fn(element_ptr) };
            }
            self.drop_fn = Some(drop_fn);
        }
    }

    /// Used to drop the internal buffer and all the contained values.
    ///
    /// Must be manually called, as the [`UnsafeBlob`] can't implement Drop
    /// due to not keeping track of length and capacity itself.
    ///
    /// # Parameters
    /// - `len`: The true current length of the [`UnsafeBlob`], shouldn't exceed the capacity.
    /// - `capacity`: The true current capacity of the [`UnsafeBlob`].
    ///
    /// # Safety
    /// The UnsafeBlob must be "discarded" and not used again after calling this method.
    pub unsafe fn drop(&mut self, len: usize, capacity: usize) {
        if self.is_allocated() {
            debug_assert!(
                capacity > 0,
                "Buffer was allocated but capacity doesn't match."
            );

            self.clear_buffer(len);
            let arr_layout = array_layout(&self.item_layout, capacity).unwrap();
            std::alloc::dealloc(self.data.unwrap().as_ptr(), arr_layout);
        }
    }

    /// Drops the last element of the [`UnsafeBlob`].
    ///
    /// # Safety
    /// `last_index` must be that of the last element in the array.
    /// After calling this method that element must not be used again unless reinitialized.
    pub unsafe fn drop_last_element(&mut self, last_index: usize) {
        if let Some(drop_fn) = self.drop_fn {
            // Set `self.drop_fn` to `None` before dropping any values to avoid double dropping values in case of an unwind.
            self.drop_fn = None;
            // SAFETY: It's safe to `promote()` the pointer, as it will be left unreachable.
            let element_ptr = self.get_unchecked_mut(last_index).promote();
            // SAFETY: `element` was stored inside the `UnsafeBlob`, so it's type must match that of `drop_fn`.
            unsafe { drop_fn(element_ptr) };
            self.drop_fn = Some(drop_fn);
        }
    }

    pub unsafe fn put_unchecked(&mut self, index: usize, value: OwnPtr<'_>) {
        todo!();
    }

    pub unsafe fn replace_unchecked(&mut self, index: usize, last_index: OwnPtr<'_>) {
        todo!();
    }

    pub unsafe fn swap_remove_unchecked(&mut self, index: usize, last_index: usize) -> OwnPtr<'_> {
        todo!();
    }

    pub unsafe fn swap_remove_drop_unchecked(&mut self, index: usize, last_index: usize) {
        todo!();
    }
}
