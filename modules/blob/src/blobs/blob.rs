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

    /// Constructs a new, empty `Blob` for the type `T`.
    ///
    /// The `Blob` will be lazily allocated untill a value is stored inside of it.
    #[inline]
    pub const fn new<T: Sized>() -> Result<Blob> {
        let layout = Layout::new::<T>();
        if layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        // SAFETY: We just checked that `T` is not a `ZST`.
        Ok(unsafe { Self::new_unchecked::<T>() })
    }

    /// Constructs a new, empty `Blob` for the type `T`.
    ///
    /// The `Blob` will be lazily allocated untill a value is stored inside of it.
    ///
    /// # Safety:
    /// The caller must ensure that `T` is not a `ZST`.
    #[inline]
    pub const unsafe fn new_unchecked<T: Sized>() -> Blob {
        // SAFETY: caller ensures that the stored value is not a `ZST`.
        // SAFETY: the `layout` and `drop_fn` are coming from a valid Rust type.
        Self::with_layout_unchecked(Layout::new::<T>(), fulmen_ptr::get_drop_fn::<T>())
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
    /// - `layout.size()` > `0`
    /// - `layout` matches that of the values being stored inside of the Blob and has propper alignement.
    /// This also implies that the `layout` matches that of the values passed to `drop_fn`.
    /// - `drop_fn` should be safe to call with any value stored inside the Blob,
    /// as long as the `drop_fn` corresponds to the errased type of the stored values.
    #[inline]
    pub const unsafe fn with_layout_unchecked(
        layout: Layout,
        drop_fn: Option<fulmen_ptr::DropFn>,
    ) -> Blob {
        // SAFETY: caller ensures that the stored value is not a `ZST`.
        // SAFETY: caller ensures the validity of `layout` and `drop_fn`.
        Self {
            data: unsafe { UnsafeBlob::with_layout_unchecked(layout, drop_fn) },
        }
    }

    /// Constructs a new `Blob` from the specified value of type `T`.
    #[inline]
    pub fn from<T: Sized>(value: T) -> Result<Blob> {
        let layout = Layout::new::<T>();
        if layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        // SAFETY: We just checked that `T` is not a `ZST`.
        Ok(unsafe { Self::from_unchecked::<T>(value) })
    }

    /// Constructs a new `Blob` from the specified value of type `T`.
    ///
    /// # Safety:
    /// The caller must ensure that `T` is not a `ZST`.
    #[inline]
    pub unsafe fn from_unchecked<T: Sized>(value: T) -> Blob {
        // SAFETY: caller ensures that the stored value is not a `ZST`.
        // SAFETY: the `layout` and `drop_fn` are coming from a valid Rust type.
        let mut blob = Self {
            data: UnsafeBlob::with_capacity_unchecked(
                Layout::new::<T>(),
                fulmen_ptr::get_drop_fn::<T>(),
                Self::CAPACITY,
            ),
        };

        OwnPtr::from(value, |ptr| {
            unsafe {
                // SAFETY: `ptr` is valid for the length of this scope.
                blob.data.initialize_unchecked(Self::INDEX, ptr);
            }
        });

        blob
    }

    /// Wether or not the `Blob` has been allocated.
    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.data.is_allocated()
    }

    /// The [`Layout`] of the values stored inside the `Blob`.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.data.item_layout()
    }

    /// Checks the layout of the type parameter `T` against `Blob::layout()`.
    #[inline]
    fn check_layout<T>(&self) -> bool {
        let t_layout = Layout::new::<T>();
        self.layout().size() == t_layout.size() && self.layout().align() == t_layout.align()
    }

    /// Replaces the value stored inside of the `Blob`, dropping the old value.
    ///
    /// If the `Blob` didn't hold a previous value, it will initialize with the passed value.
    #[inline]
    pub fn replace<T: Sized>(&mut self, value: T) {
        assert!(
            {
                let _l = Layout::new::<T>();
                self.layout().size() == _l.size() && self.layout().align() == _l.align()
            },
            "Layout of values stored on a Blob should match the Blob's layout"
        );

        if !self.is_allocated() {
            // SAFETY: `Blob` was not allocated
            unsafe {
                self.data
                    .alloc_buffer(NonZeroUsize::new_unchecked(Self::CAPACITY))
            };
            OwnPtr::from(value, |ptr| {
                unsafe {
                    // SAFETY: `ptr` is valid for the length of this scope.
                    self.data.initialize_unchecked(Self::INDEX, ptr);
                }
            });
        } else {
            OwnPtr::from(value, |ptr| {
                unsafe {
                    // SAFETY: `ptr` is valid for the length of this scope.
                    self.data.replace_unchecked(Self::INDEX, ptr);
                }
            });
        }
    }

    /// Replaces the value stored inside of the `Blob`, dropping the old value.
    ///
    /// # Safety:
    /// The caller must ensure:
    /// - the `Blob` had been previouslly allocated.
    /// - that `T` is not a `ZST`.
    #[inline]
    pub fn replace_unchecked<T: Sized>(&mut self, value: T) {
        assert!(
            {
                let _l = Layout::new::<T>();
                self.layout().size() == _l.size() && self.layout().align() == _l.align()
            },
            "Layout of values stored on a Blob should match the Blob's layout"
        );
        // SAFETY: caller ensures `Blob` was allocated and that `value` is not a `ZST`.
        OwnPtr::from(value, |ptr| {
            unsafe {
                // SAFETY: `ptr` is valid for the length of this scope.
                self.data.replace_unchecked(Self::INDEX, ptr);
            }
        });
    }

    /// Gets the [`Ptr`] to the start of the underlying buffer.
    #[inline]
    pub fn get_ptr(&self) -> Option<Ptr<'_>> {
        self.data.get_ptr()
    }

    /// Gets the [`Ptr`] to the start of the underlying buffer.
    ///
    /// # Safety:
    /// The caller must ensure that the `UnsafeBlob` is allocated.
    #[inline]
    pub unsafe fn get_ptr_unchecked(&self) -> Ptr<'_> {
        self.data.get_ptr_unchecked()
    }

    /// Gets the [`PtrMut`] to the start of the underlying buffer.
    #[inline]
    pub fn get_ptr_mut(&self) -> Option<PtrMut<'_>> {
        self.data.get_ptr_mut()
    }

    /// Gets the [`PtrMut`] to the start of the underlying buffer.
    ///
    /// # Safety:
    /// The caller must ensure that the `UnsafeBlob` is allocated.
    #[inline]
    pub unsafe fn get_ptr_mut_unchecked(&self) -> PtrMut<'_> {
        self.data.get_ptr_mut_unchecked()
    }

    pub fn downcast_ref<T: Sized>(&self) -> Result<&T> {
        if self.is_allocated() {
            if self.check_layout::<T>() {
                // SAFETY: We checked layouts, but we can't be 100% sure
                Ok(unsafe { self.get_ptr_unchecked().deref::<T>() })
            } else {
                Err(Error::LayoutMistmatch)
            }
        } else {
            Err(Error::UninitializedBlob)
        }
    }

    pub unsafe fn downcast_ref_unchecked<T: Sized>(&self) -> &T {
        // SAFETY: Caller ensures the errased `type` and `layout` match.
        unsafe { self.get_ptr_unchecked().deref::<T>() }
    }

    pub fn downcast_mut<T: Sized>(&mut self) -> Result<&mut T> {
        if self.is_allocated() {
            if self.check_layout::<T>() {
                // SAFETY: We checked layouts, but we can't be 100% sure
                Ok(unsafe { self.get_ptr_mut_unchecked().deref_mut::<T>() })
            } else {
                Err(Error::LayoutMistmatch)
            }
        } else {
            Err(Error::UninitializedBlob)
        }
    }

    pub unsafe fn downcast_mut_unchecked<T: Sized>(&mut self) -> &mut T {
        // SAFETY: Caller ensures the errased `type` and `layout` match.
        unsafe { self.get_ptr_mut_unchecked().deref_mut::<T>() }
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        let len = if self.is_allocated() { 1 } else { 0 };

        unsafe {
            self.data.drop(len, Self::CAPACITY);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Blob;
    use core::alloc::Layout;
    use std::str::FromStr;

    #[test]
    fn new_blob() {
        let _ = Blob::new::<&str>().unwrap();
        let blob = Blob::new::<u32>().unwrap();
        assert_eq!(false, blob.is_allocated());
    }

    #[test]
    #[should_panic]
    fn new_blob_zst() {
        let _ = Blob::new::<()>().unwrap();
    }

    struct PanicOnDrop(u8);
    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!("A 'PanicOnDrop' was dropped");
        }
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn new_unchecked_blob_zst() {
        let _ = unsafe { Blob::new_unchecked::<()>() };
    }

    #[test]
    #[cfg(debug_assertions)]
    fn with_layout_unchecked() {
        let layout = Layout::new::<u32>();

        let blob = unsafe { Blob::with_layout_unchecked(layout, None) };
        assert_eq!(false, blob.is_allocated());
    }

    const LOCALHOST_IP: &str = "127.0.0.1";

    #[test]
    fn from_struct() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(_localhost).unwrap();

        assert_eq!(true, blob.is_allocated());
        assert_eq!(Layout::new::<Ipv4Addr>(), blob.layout());
    }

    #[test]
    fn new_no_panic() {
        let blob = Blob::new::<PanicOnDrop>().unwrap();
        drop(blob);
    }

    #[test]
    #[should_panic(expected = "A 'PanicOnDrop' was dropped")]
    fn new_panic_on_drop() {
        let mut blob = Blob::new::<PanicOnDrop>().unwrap();
        blob.replace(PanicOnDrop(0_u8));
        drop(blob);
    }

    #[test]
    fn downcast() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(_localhost).unwrap();

        assert_eq!(true, blob.is_allocated());

        let layout = Layout::new::<Ipv4Addr>();
        assert_eq!(layout, blob.layout());

        // Recover value
        let _ = blob.downcast_ref::<Ipv4Addr>().unwrap();
        let data = unsafe { blob.downcast_ref_unchecked::<Ipv4Addr>() };
        assert_eq!(data.octets(), _localhost.octets());
    }

    #[test]
    #[should_panic]
    fn downcast_uninit() {
        let blob = Blob::new::<u32>().unwrap();
        let _: &u32 = blob.downcast_ref::<u32>().unwrap();
    }

    #[test]
    #[should_panic]
    fn downcast_type_missmatch() {
        use std::net::Ipv4Addr;
        let blob = Blob::new::<u32>().unwrap();
        let _ = blob.downcast_ref::<Ipv4Addr>().unwrap();
    }
}
