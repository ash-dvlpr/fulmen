use crate::{Error, Result};

use core::{alloc::Layout, marker::PhantomData, mem, ptr::NonNull, slice};

pub struct Blob {
    data: Option<NonNull<u8>>,
    layout: Layout,
    _marker: PhantomData<u8>,
}

impl Blob {
    #[inline]
    pub const fn new<T: ?Sized + Copy>() -> Result<Blob> {
        if mem::needs_drop::<T>() {
            return Err(Error::CantDropType);
        }

        let layout = Layout::new::<T>();
        Self::with_layout(layout)
    }

    /// # Safety
    /// - Make sure the layout size for T is `> 0` and type T doesn't implement Drop, or you handle it elsewere.
    #[inline]
    pub const unsafe fn new_unchecked<T: Sized>() -> Blob {
        let layout = Layout::new::<T>();
        Self::with_layout_unchecked(layout)
    }

    #[inline]
    pub const fn with_layout(layout: Layout) -> Result<Blob> {
        if layout.size() == 0 {
            return Err(Error::ZeroSizedLayout);
        }

        Ok(unsafe { Self::with_layout_unchecked(layout) })
    }

    /// # Safety
    /// - Make sure `layout.size` > 0 and is propperly aligned
    #[inline]
    pub const unsafe fn with_layout_unchecked(layout: Layout) -> Blob {
        debug_assert!(layout.size() > 0, "Size must be non zero");
        debug_assert!(layout.align() > 0, "Alignment must be non zero");
        debug_assert!(
            layout.align().is_power_of_two(),
            "Alignment must be a power of two"
        );

        Blob {
            layout,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn from<T: ?Sized + Copy>(value: T) -> Result<Blob> {
        let mut blob = Self::new::<T>()?;

        unsafe {
            // SAFETY: We just checked for zero sized layouts on `new()`.
            blob.init_unchecked();
            // SAFETY: We assume we got passed a valid reference
            blob.replace_bytes_unchecked(&value as *const _ as *const u8);
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
            // SAFETY: We just checked for zero sized layouts on `new()`.
            blob.init_unchecked();
            // SAFETY: We assume we got passed a valid reference
            blob.replace_bytes_unchecked(bytes.as_ptr());
        }

        Ok(blob)
    }

    // TODO: fn from_bytes_unchecked

    /// # Safety
    /// - `source` must be a valid pointer with the same Layout as the Blob.
    #[inline]
    pub fn replace_bytes(&mut self, source: &[u8]) {
        debug_assert!(
            self.layout.size() == source.len(),
            "Length of a byte slice being stored on a Blob should match the Blob's layout size"
        );

        if self.data.is_none() {
            unsafe {
                self.init_unchecked();
            }
        }

        unsafe {
            self.replace_bytes_unchecked(source.as_ptr());
        }
    }

    /// # Safety
    /// - The blob must have been already allocated. This can be checked with [`Self::is_init()`].
    /// - `source` must be a valid pointer and the data it points to should have the same Layout as the Blob.
    #[inline]
    unsafe fn replace_bytes_unchecked(&mut self, source: *const u8) {
        std::ptr::copy_nonoverlapping::<u8>(
            source,
            self.data.unwrap().as_ptr(),
            self.layout.size(),
        );
    }

    /// # Safety
    /// See [`GlobalAlloc::alloc`].
    #[inline]
    unsafe fn init_unchecked(&mut self) {
        self.data = Some(Self::create_buffer_unchecked(self.layout));
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
        self.layout
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        if let Some(ptr) = self.data {
            unsafe { slice::from_raw_parts(ptr.as_ptr(), self.layout.size()) }
        } else {
            &[]
        }
    }

    #[inline]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        if let Some(ptr) = self.data {
            unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), self.layout.size()) }
        } else {
            &mut []
        }
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if let Some(ptr) = self.data {
            unsafe { std::alloc::dealloc(ptr.as_ptr(), self.layout) }
        }
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
    fn new_blob_unchecked_zst() {
        let _ = unsafe { Blob::new_unchecked::<()>() };
    }

    #[test]
    #[should_panic]
    fn new_blob_invalid_types() {
        // let _ = Blob::new::<()>().unwrap();
        let _ = unsafe { Blob::new_unchecked::<()>() };
    }

    #[test]
    fn with_layout() {
        let layout = Layout::new::<u32>();
        let blob = Blob::with_layout(layout).unwrap();

        assert_eq!(false, blob.is_init());
        assert_eq!(0, blob.bytes().len());
    }

    #[test]
    #[should_panic]
    fn with_invalid_layout() {
        let layout = unsafe { Layout::from_size_align_unchecked(3, 3) };
        let _ = Blob::with_layout(layout).unwrap();
    }

    const LOCALHOST_IP: &str = "127.0.0.1";

    #[test]
    fn from_struct() {
        use std::net::Ipv4Addr;
        let localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from(localhost).unwrap();

        assert_eq!(true, blob.is_init());
        assert_eq!(blob.layout().size(), blob.bytes().len());

        // TODO: Recover value
    }

    #[test]
    fn from_bytes() {
        use std::net::Ipv4Addr;
        let localhost = Ipv4Addr::from_str(LOCALHOST_IP).unwrap();

        let blob = Blob::from_bytes(localhost.octets()).unwrap();

        assert_eq!(true, blob.is_init());
        assert_eq!(blob.layout().size(), blob.bytes().len());

        // TODO: Recover value
    }
}
