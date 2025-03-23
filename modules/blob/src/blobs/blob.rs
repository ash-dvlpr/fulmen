use fulmen_ptr::{OwnPtr, Ptr, PtrMut};

use super::UnsafeBlob;
use crate::{Error, Result};

use core::{alloc::Layout, num::NonZeroUsize};

/// Type erased data storage.
pub struct Blob {
    data: UnsafeBlob,
}

impl Blob {
    const INDEX: usize = 0;
    const CAPACITY: usize = 1;

    // region: Constructors
    /// Constructs a new, empty `Blob` for the type `T`.
    ///
    /// The `Blob` will be lazily allocated untill a value is stored inside of it.
    #[inline]
    pub const fn new<T: Sized>() -> Blob {
        // SAFETY: the `layout` and `drop_fn` are coming from a valid Rust type.
        unsafe { Self::with_layout_unchecked(Layout::new::<T>(), fulmen_ptr::get_drop_fn::<T>()) }
    }

    /// Constructs a new, empty `Blob` for the specified `layout`.
    ///
    /// The `Blob` will be lazily allocated untill a value is stored inside of it.
    ///
    /// If the `drop_fn` is `None`, values will be leaked in the case that their errased type implements [`Drop`].
    /// This should be set to `None` based on [`core::mem::needs_drop`].
    ///
    /// # Safetly
    /// The caller must ensure the following:
    /// - `layout` matches that of the values being stored inside of the Blob and has propper alignement.
    ///   This also implies that the `layout` matches that of the values passed to `drop_fn`.
    /// - `drop_fn` should be safe to call with any value stored inside the Blob,
    ///   as long as the `drop_fn` corresponds to the errased type of the stored values.
    #[inline]
    pub const unsafe fn with_layout_unchecked(
        layout: Layout,
        drop_fn: Option<fulmen_ptr::DropFn>,
    ) -> Blob {
        // SAFETY: caller ensures the validity of `layout` and `drop_fn`.
        Self {
            data: unsafe { UnsafeBlob::with_layout_unchecked(layout, drop_fn) },
        }
    }

    /// Constructs a new `Blob` from the specified value of type `T`.
    #[inline]
    pub fn from<T: Sized>(value: T) -> Blob {
        // SAFETY: the `layout` and `drop_fn` are coming from a valid Rust type.
        let mut blob = Self {
            data: unsafe {
                UnsafeBlob::with_capacity_unchecked(
                    Layout::new::<T>(),
                    fulmen_ptr::get_drop_fn::<T>(),
                    Self::CAPACITY,
                )
            },
        };

        OwnPtr::from(value, |ptr| {
            unsafe {
                // SAFETY: `ptr` is valid for the length of this scope.
                blob.data.initialize_unchecked(Self::INDEX, ptr);
            }
        });

        blob
    }

    // endregion

    // region: Propperties

    /// Wether or not the `Blob` has been allocated.
    #[inline]
    pub fn has_value(&self) -> bool {
        self.data.is_allocated()
    }

    /// The [`Layout`] of the values stored inside the `Blob`.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.data.item_layout()
    }

    /// Checks the layout of the type parameter `T` against [`Self::layout`].
    #[inline]
    fn check_layout<T>(&self) -> bool {
        let t_layout = Layout::new::<T>();
        self.layout().size() == t_layout.size() && self.layout().align() == t_layout.align()
    }

    // endregion

    // region: Insert & Replace
    /// Inserts the `value` stored inside of the `Blob`, dropping the old value.
    ///
    /// If the `Blob` didn't hold a previous value, the buffer will be initialized.
    #[inline]
    pub fn insert<T: Sized>(&mut self, value: T) {
        assert!(
            {
                let _l = Layout::new::<T>();
                self.layout().size() == _l.size() && self.layout().align() == _l.align()
            },
            "Layout of values stored on a Blob should match the Blob's layout"
        );

        OwnPtr::from(value, |ptr| {
            // SAFETY: We just checked for matching layouts
            unsafe { self.insert_data(ptr) }
        });
    }

    /// Inserts the value referenced by `ptr` inside of the `Blob`, dropping the old value.
    ///
    /// If the `Blob` didn't hold a previous value, the buffer will be initialized.
    ///
    /// # Safety
    /// The caller must ensure that [`Self::layout`] matches the layout of the value referended by `ptr`.
    #[inline]
    pub unsafe fn insert_data(&mut self, ptr: OwnPtr<'_>) {
        if !self.has_value() {
            unsafe {
                // SAFETY: `Blob` was not allocated
                self.data
                    .alloc_buffer(NonZeroUsize::new_unchecked(Self::CAPACITY));
                // SAFETY: `ptr` is valid for the length of this scope.
                self.data.initialize_unchecked(Self::INDEX, ptr);
            }
        } else {
            unsafe {
                // SAFETY: `ptr` is valid for the length of this scope.
                self.data.replace_unchecked(Self::INDEX, ptr);
            }
        }
    }

    // endregion

    // region: Data access
    /// Gets the [`Ptr`] to the start of the underlying buffer.
    #[inline]
    pub fn get_ptr(&self) -> Option<Ptr<'_>> {
        if self.has_value() {
            // SAFETY: We just checked that the buffer is allocated
            Some(unsafe { self.data.get_ptr() })
        } else {
            None
        }
    }

    /// Gets the [`Ptr`] to the start of the underlying buffer.
    ///
    /// # Safety:
    /// The caller must ensure that the `UnsafeBlob` is allocated.
    #[inline]
    pub unsafe fn get_ptr_unchecked(&self) -> Ptr<'_> {
        // SAFETY: The caller must ensure that the `UnsafeBlob` is allocated.
        unsafe { self.data.get_ptr() }
    }

    /// Gets the [`PtrMut`] to the start of the underlying buffer.
    #[inline]
    pub fn get_ptr_mut(&self) -> Option<PtrMut<'_>> {
        if self.has_value() {
            // SAFETY: We just checked that the buffer is allocated
            Some(unsafe { self.data.get_ptr_mut() })
        } else {
            None
        }
    }

    /// Gets the [`PtrMut`] to the start of the underlying buffer.
    ///
    /// # Safety:
    /// The caller must ensure that the `UnsafeBlob` is allocated.
    #[inline]
    pub unsafe fn get_ptr_mut_unchecked(&self) -> PtrMut<'_> {
        // SAFETY: The caller must ensure that the `UnsafeBlob` is allocated.
        unsafe { self.data.get_ptr_mut() }
    }

    pub fn downcast_ref<T: Sized>(&self) -> Result<&T> {
        if self.check_layout::<T>() {
            if self.has_value() {
                // SAFETY: We checked layouts, but we can't be 100% sure
                Ok(unsafe { self.get_ptr_unchecked().deref::<T>() })
            } else {
                Err(Error::None)
            }
        } else {
            Err(Error::LayoutMistmatch)
        }
    }

    pub unsafe fn downcast_ref_unchecked<T: Sized>(&self) -> &T {
        // SAFETY: Caller ensures the errased `type` and `layout` match.
        unsafe { self.get_ptr_unchecked().deref::<T>() }
    }

    pub fn downcast_mut<T: Sized>(&mut self) -> Result<&mut T> {
        if self.check_layout::<T>() {
            if self.has_value() {
                // SAFETY: We checked layouts, but we can't be 100% sure
                Ok(unsafe { self.get_ptr_mut_unchecked().deref_mut::<T>() })
            } else {
                Err(Error::None)
            }
        } else {
            Err(Error::LayoutMistmatch)
        }
    }

    pub unsafe fn downcast_mut_unchecked<T: Sized>(&mut self) -> &mut T {
        // SAFETY: Caller ensures the errased `type` and `layout` match.
        unsafe { self.get_ptr_mut_unchecked().deref_mut::<T>() }
    }

    // endregion
}

impl Drop for Blob {
    fn drop(&mut self) {
        let len = if self.has_value() { 1 } else { 0 };

        unsafe {
            self.data.drop(len, Self::CAPACITY);
        }
    }
}

#[cfg(test)]
mod tests {
    use fulmen_ptr::OwnPtr;

    use super::Blob;
    use core::alloc::Layout;
    use std::str::FromStr;

    #[test]
    fn new_blob() {
        let blob = Blob::new::<&str>();
        assert_eq!(false, blob.has_value());
        let blob = Blob::new::<u32>();
        assert_eq!(false, blob.has_value());
    }

    #[test]
    fn new_blob_zst() {
        let blob = Blob::new::<()>();
        assert_eq!(false, blob.has_value())
    }

    #[test]
    #[cfg(debug_assertions)]
    fn with_layout_unchecked() {
        let layout = Layout::new::<u32>();

        let blob = unsafe { Blob::with_layout_unchecked(layout, None) };
        assert_eq!(false, blob.has_value());
    }

    const LOCALHOST_IP: &str = "127.0.0.1";

    #[test]
    fn from_struct() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(_localhost);

        assert_eq!(true, blob.has_value());
        assert_eq!(Layout::new::<Ipv4Addr>(), blob.layout());
    }

    struct PanicOnDrop;
    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!("A 'PanicOnDrop' was dropped");
        }
    }

    #[test]
    fn new_no_drop() {
        _ = Blob::new::<PanicOnDrop>();
    }

    #[test]
    #[should_panic(expected = "A 'PanicOnDrop' was dropped")]
    fn new_drop() {
        let mut blob = Blob::new::<PanicOnDrop>();
        blob.insert(PanicOnDrop);
    }

    #[test]
    fn replace_ptr() {
        let mut blob = Blob::from(100_u32);
        assert_eq!(&100, blob.downcast_ref::<u32>().unwrap());

        OwnPtr::from(0_u32, |ptr| unsafe {
            blob.insert_data(ptr);
        });
        assert_ne!(&100, blob.downcast_ref::<u32>().unwrap());
        assert_eq!(&0, blob.downcast_ref::<u32>().unwrap());
    }

    #[test]
    fn downcast() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(_localhost);

        assert_eq!(true, blob.has_value());

        let layout = Layout::new::<Ipv4Addr>();
        assert_eq!(layout, blob.layout());

        // Recover value
        _ = blob.downcast_ref::<Ipv4Addr>().unwrap();
        let data = unsafe { blob.downcast_ref_unchecked::<Ipv4Addr>() };
        assert_eq!(data.octets(), _localhost.octets());
    }

    #[test]
    fn downcast_zst() {
        let blob = Blob::from(());
        assert_eq!(true, blob.has_value());

        // Recover value
        _ = blob.downcast_ref::<()>().unwrap();
        _ = unsafe { blob.downcast_ref_unchecked::<()>() };
    }

    #[test]
    #[should_panic(expected = "None")]
    fn downcast_uninit() {
        let blob = Blob::new::<u32>();
        _ = blob.downcast_ref::<u32>().unwrap();
    }

    #[test]
    #[should_panic(expected = "LayoutMistmatch")]
    fn downcast_type_missmatch() {
        use std::net::Ipv4Addr;
        let blob = Blob::new::<u32>();
        _ = blob.downcast_ref::<Ipv4Addr>().unwrap();
    }
}
