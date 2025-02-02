use super::UnsafeBlob;
use crate::{Error, Result};

use core::{alloc::Layout, marker::PhantomData, mem, ptr::{self, NonNull}, slice};


/// Type erased data storage.
pub struct Blob {
    data: UnsafeBlob,
}

impl Blob {
    const CAPACITY: usize = 1;

    /// # Safety
    /// The type `T` doesn't implement `Drop`.
    #[inline]
    pub const fn new<T: Sized>() -> Result<Blob> {
        let layout = Layout::new::<T>();
        let mut blob = Self::with_layout(layout);

        if mem::needs_drop::<T>() && blob.is_ok() {
           &blob.unwrap().drop_fn = Some(
                ptr::drop_in_place::<T>
           );
        }

        blob
    }

    /// # Safety
    /// This function is unsafe as it does not verify the preconditions from [`Blob::new`].
    #[inline]
    pub const unsafe fn new_unchecked<T: Sized>() -> Blob {
        let layout = Layout::new::<T>();
        let drop_fn = None;
        Self::with_layout_unchecked(layout, drop_fn)
    }

    #[inline]
    pub const fn with_layout(layout: Layout) -> Result<Blob> {
        if layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        // TODO: SAFETY: ---
        Ok(unsafe { Self::with_layout_unchecked(layout, None) })
    }

    /// # Safety
    /// This function is unsafe as it does not verify the preconditions from [`Blob::with_layout`].
    #[inline]
    pub const unsafe fn with_layout_unchecked(layout: Layout, drop_fn: Option<unsafe fn(*mut u8)>) -> Blob {
        debug_assert!(layout.size() > 0, "Size must be non zero");

        Blob {
            data: None,
            drop_fn: drop_fn,
            item_layout: layout,
            _marker: PhantomData,
        }
    }

    pub fn from<T: Sized>(value: T) -> Result<Blob> {
        let mut blob = Self::new::<T>()?;

        unsafe {
            // SAFETY: We just checked for zero sized layouts on `new()`.
            blob.init_unchecked();
            // SAFETY: We assume we got passed a valid reference
            blob.replace_bytes_unchecked_internal(&value as *const _ as *const u8);
        }

        Ok(blob)
    }

    // TODO: fn from_unchecked

    #[inline]
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Blob> {
        let bytes = bytes.as_ref();
        let layout = Layout::array::<u8>(bytes.len())?;

        let mut blob = Self::with_layout(layout)?;
        unsafe {
            // SAFETY: We just checked for zero sized layouts on `with_layout()`.
            blob.init_unchecked();
            // SAFETY: We assume we got passed a valid reference
            blob.replace_bytes_unchecked(bytes);
        }

        Ok(blob)
    }

    // TODO: fn from_bytes_unchecked

    pub fn replace<T: Sized>(&mut self, value: T) {
        assert!(
            {
                let _l = Layout::new::<T>();
                self.item_layout.size() == _l.size() && self.item_layout.align() == _l.align()
            },
            "Layout of values stored on a Blob should match the Blob's layout"
        );

        // TODO: behavior should be that of take()
        unsafe {
            // TODO: SAFETY: ---
            self.replace_unchecked(value);
        }
    }

    /// Safety:
    /// This function is unsafe as it does not verify the preconditions from [`Blob::replace`].
    /// Aditionally:
    ///
    /// - The
    pub unsafe fn replace_unchecked<T: Sized>(&mut self, value: T) {
        debug_assert!(
            {
                let _l = Layout::new::<T>();
                self.item_layout.size() == _l.size() && self.item_layout.align() == _l.align()
            },
            "Layout of values stored on a Blob should match the Blob's layout"
        );

        // TODO: behavior should be that of take()
        // TODO: SAFETY: ---
        self.replace_bytes_unchecked_internal(&value as *const _ as *const u8);
    }

    /// # Safety
    /// - `source` must be a valid pointer and the data it points to should have the same `Layout` as the Blob.
    #[inline]
    pub fn replace_bytes(&mut self, source: &[u8]) {
        assert!(
            self.item_layout.size() == source.len(),
            "Length of a byte slice being stored on a Blob should match the Blob's layout size"
        );

        self.init();
        unsafe {
            self.replace_bytes_unchecked_internal(source.as_ptr());
        }
    }

    /// # Safety
    /// This function is unsafe as it does not verify the preconditions from [`Blob::replace_bytes`], and Additionally:
    ///
    /// - The blob must have been already allocated. This can be checked with [`Self::is_init()`].
    #[inline]
    pub unsafe fn replace_bytes_unchecked(&mut self, source: &[u8]) {
        debug_assert!(
            self.item_layout.size() == source.len(),
            "Length of a byte slice being stored on a Blob should match the Blob's layout size"
        );

        self.replace_bytes_unchecked_internal(source.as_ptr());
    }

    /// # Safety
    /// This function is unsafe as it does not verify the preconditions from [`Blob::replace_bytes`], and Additionally:
    ///
    /// - The blob must have been already allocated. This can be checked with [`Self::is_init()`].
    #[inline]
    unsafe fn replace_bytes_unchecked_internal(&mut self, source: *const u8) {
        debug_assert!(
            self.data.is_some(),
            "Blob must be initialized before replacing it's contents"
        );

        std::ptr::copy_nonoverlapping::<u8>(
            source,
            self.data.unwrap().as_ptr(),
            self.item_layout.size(),
        );
    }

    #[inline]
    fn init(&mut self) {
        if self.data.is_none() {
            unsafe {
                self.init_unchecked();
            }
        }
    }

    /// # Safety
    /// See [`GlobalAlloc::alloc`].
    #[inline]
    unsafe fn init_unchecked(&mut self) {
        self.data = Some(Self::create_buffer_unchecked(self.item_layout));
    }

    /// # Safety
    /// See [`GlobalAlloc::alloc`].
    #[inline]
    unsafe fn create_buffer_unchecked(layout: Layout) -> NonNull<u8> {
        let ptr = std::alloc::alloc(layout);
        NonNull::new_unchecked(ptr)
    }

    /// Indicates whether or not the Blob has already been allocated.
    #[inline]
    pub fn is_init(&self) -> bool {
        self.data.is_some()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.item_layout
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        if let Some(ptr) = self.data {
            unsafe { slice::from_raw_parts(ptr.as_ptr(), self.item_layout.size()) }
        } else {
            &[]
        }
    }

    #[inline]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        if let Some(ptr) = self.data {
            unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), self.item_layout.size()) }
        } else {
            &mut []
        }
    }

    #[inline]
    pub fn downcast<T>(&self) -> Result<&T> {
        if let Some(data) = self.data {
            let t_layout = Layout::new::<T>();
            if (self.item_layout.size() == t_layout.size() && self.item_layout.align() == t_layout.align()) {
                Ok(unsafe { self.downcast_unchecked() })
            } else {
                Err(Error::LayoutMistmatch)
            }
        }
        else { 
            Err(Error::UninitializedBlob)
        }
    }

    /// Safety:
    /// This function is unsafe as it does not verify the preconditions from [`Blob::downcast`].
    #[inline]
    pub unsafe fn downcast_unchecked<T>(&self) -> &T {
        debug_assert!(self.data.is_some(), "Blob should be initialized");
        & *(self.data.unwrap().as_ptr() as *const T)
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if let Some(ptr) = self.data {
            if let Some(drop_fn) = self.drop_fn {
                self.drop_fn = None;
                unsafe { drop_fn(ptr.as_ptr()); }
                self.drop_fn = Some(drop_fn);
            }

            unsafe { std::alloc::dealloc(ptr.as_ptr(), self.item_layout) }
            self.data = None;
        }
        self.drop_fn = None;
    }
}

impl AsRef<[u8]> for Blob {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}

impl AsMut<[u8]> for Blob {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        self.bytes_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::Blob;
    use core::alloc::Layout;
    use std::str::FromStr;

    #[test]
    fn new_blob() {
        let _ = Blob::new::<u32>().unwrap();
        let _ = Blob::new::<&str>().unwrap();
    }

    #[test]
    #[should_panic]
    fn new_blob_zst() {
        let _ = Blob::new::<()>().unwrap();
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
        let blob = Blob::with_layout(layout).unwrap();

        assert_eq!(false, blob.is_init());
        assert_eq!(0, blob.bytes().len());
    }

    const LOCALHOST_IP: &str = "127.0.0.1";

    #[test]
    fn from_struct() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(_localhost).unwrap();

        assert_eq!(true, blob.is_init());
        assert_eq!(blob.layout().size(), blob.bytes().len());

        // TODO: Recover value
    }

    #[test]
    fn from_bytes() {
        use std::net::Ipv4Addr;
        let _localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from_bytes(_localhost.octets()).unwrap();

        assert_eq!(true, blob.is_init());
        assert_eq!(blob.layout().size(), blob.bytes().len());

        // TODO: Recover value
    }
}
