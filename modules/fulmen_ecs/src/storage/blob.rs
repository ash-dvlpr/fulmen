use std::{alloc::Layout, any::Any, mem, ptr, ptr::NonNull};

pub struct BlobData {
    // TODO: change internal data store to a propper BLOB
    data: NonNull<dyn Any>,
    // buf: NonNull<u8>, // BLOB
    // data_layout: alloc::Layout,
    // len: usize,
    // cap: usize,
}

impl BlobData {
    pub unsafe fn new(data_layout: Layout) -> Self {
        Self {
            data: dangling_from_layout(data_layout),
        }
    }

    #[inline]
    pub fn get(&self) -> &NonNull<dyn Any> {
        &self.data
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut NonNull<dyn Any> {
        &mut self.data
    }
}

/// Creates a danging pointer that conforms to the specified [`Layout`] alignment.
/// # See:
/// * [`NonNull::dangling`]
const fn dangling_from_layout(layout: Layout) -> NonNull<u8> {
    debug_assert!(layout.align() > 0, "Alignment must be non zero");
    debug_assert!(
        layout.align().is_power_of_two(),
        "Alignment must be a power of two"
    );
    // SAFETY: mem::align_of() returns a non-zero usize which is then casted
    // to a *mut T. Therefore, `ptr` is not null and the conditions for
    // calling new_unchecked() are respected.
    unsafe {
        // Note (`0 + align` as *mut T) == (`align` as *mut T)
        let ptr = ptr::null_mut::<u8>().wrapping_add(layout.align());
        NonNull::new_unchecked(ptr)
    }
}

#[cfg(test)]
mod test {
    use std::{alloc::Layout, mem, ptr::NonNull};

    #[allow(dead_code)]
    struct TestStruct(pub u32);

    #[test]
    fn test_dangling_from_layout() {
        // STD's dangling ptr
        {
            let non_null = NonNull::<TestStruct>::dangling();
            assert_ne!((non_null.as_ptr() as usize), 0);
            assert_eq!((non_null.as_ptr() as usize), mem::align_of::<TestStruct>());
        }

        // My dangling ptr
        {
            let layout = Layout::new::<TestStruct>();
            let non_null = super::dangling_from_layout(layout);
            assert_ne!((non_null.as_ptr() as usize), 0);
            assert_eq!((non_null.as_ptr() as usize), mem::align_of::<TestStruct>());
        }
    }
}
