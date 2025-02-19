use core::alloc::Layout;
use core::ptr::{self, NonNull};

/// Creates a danging pointer that conforms to the specified [`Layout`] alignment.
/// # See:
/// * [`NonNull::dangling`]
pub const fn dangling_from_layout(layout: Layout) -> NonNull<u8> {
    debug_assert!(layout.align() > 0, "Alignment must be non zero");
    debug_assert!(
        layout.align().is_power_of_two(),
        "Alignment must be a power of two"
    );

    // SAFETY: align() is non-zero usize which is then casted
    // to a *mut T. Therefore, `ptr` is not null and the conditions for
    // calling new_unchecked() are respected.
    unsafe {
        // Note (`0 + align` as *mut T) == (`align` as *mut T)
        let ptr = ptr::null_mut::<u8>().wrapping_add(layout.align());
        NonNull::new_unchecked(ptr)
    }
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
pub fn array_layout(item_layout: &Layout, capacity: usize) -> Option<Layout> {
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
mod test {
    use super::*;
    use core::alloc::Layout;
    use core::mem;

    #[allow(dead_code)]
    struct TestStruct(usize, u8, u8);

    macro_rules! test_dangling_from_layout_internal {
        ($ttype:ty) => {
            // STD's dangling ptr
            {
                let non_null = NonNull::<$ttype>::dangling();
                assert_ne!((non_null.as_ptr() as usize), 0);
                assert_eq!((non_null.as_ptr() as usize), mem::align_of::<$ttype>());
            }

            // My dangling ptr
            {
                let layout = Layout::new::<$ttype>();
                let non_null = super::dangling_from_layout(layout);
                assert_ne!((non_null.as_ptr() as usize), 0);
                assert_eq!((non_null.as_ptr() as usize), mem::align_of::<$ttype>());
            }
        };
    }

    #[test]
    fn test_dangling_from_layout() {
        test_dangling_from_layout_internal!(u8);
        test_dangling_from_layout_internal!(TestStruct);
        test_dangling_from_layout_internal!(Vec<TestStruct>);
        test_dangling_from_layout_internal!(());
    }

    macro_rules! test_array_layout_internal {
        ($ttype:ty) => {
            let layout = Layout::new::<$ttype>();

            let (array_layout, offset) = layout_repeat(&layout, 4).unwrap();
            debug_assert_eq!(layout.size(), offset);
            debug_assert_eq!(array_layout.size() / 4, layout.size());
        };
    }

    #[test]
    fn test_array_layout() {
        test_array_layout_internal!(u8);
        test_array_layout_internal!(TestStruct);
        test_array_layout_internal!(Vec<TestStruct>);
        test_array_layout_internal!(());
    }
}
