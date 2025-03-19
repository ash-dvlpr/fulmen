use core::marker::PhantomData;
use core::ptr::NonNull;

use core::fmt::{self, Formatter, Pointer};
use core::mem::ManuallyDrop;

pub type DropFn = unsafe fn(OwnPtr<'_>);

/// Type errased pointer that acts as `&dyn Any`.
/// Acts as a thin pointer with no metadata, without restarints to pointing to Rust data types.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Ptr<'a>(NonNull<u8>, PhantomData<&'a u8>);

/// Type errased pointer that acts as `&mut dyn Any`.
/// Acts as a thin pointer with no metadata, without restarints to pointing to Rust data types.
#[derive(Debug)]
#[repr(transparent)]
pub struct PtrMut<'a>(NonNull<u8>, PhantomData<&'a mut u8>);

/// Type errased pointer that acts as `&mut ManuallyDrop<dyn Any>`.
/// Acts as a thin pointer with no metadata, without restarints to pointing to Rust data types.
#[derive(Debug)]
#[repr(transparent)]
pub struct OwnPtr<'a>(NonNull<u8>, PhantomData<&'a mut u8>);

macro_rules! impl_ptr {
    ($ptr:ident) => {
        impl<'a> From<$ptr<'a>> for NonNull<u8> {
            fn from(ptr: $ptr<'a>) -> Self {
                ptr.0
            }
        }

        impl Pointer for $ptr<'_> {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                Pointer::fmt(&self.0, f)
            }
        }

        impl $ptr<'_> {
            /// Gets the underlying pointer, erasing the associated lifetime.
            #[inline]
            pub fn as_ptr(&self) -> *mut u8 {
                self.0.as_ptr()
            }

            /// Calculates the offset from a pointer.
            /// As the pointer is type-erased, there is no size information available. The provided
            /// `count` parameter is in raw bytes.
            ///
            /// *See also: [`ptr::offset`][ptr_offset]*
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null, or take it out of bounds for its allocation.
            /// - The value pointed by the resulting pointer must outlive the lifetime of this pointer.
            ///
            /// [ptr_offset]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
            #[inline]
            pub unsafe fn byte_offset(self, count: isize) -> Self {
                Self(
                    // SAFETY: The caller upholds safety for `offset` and ensures the result is not null.
                    unsafe { NonNull::new_unchecked(self.as_ptr().offset(count)) },
                    PhantomData,
                )
            }

            /// Calculates the offset from a pointer (convenience for `.offset(count as isize)`).
            /// As the pointer is type-erased, there is no size information available. The provided
            /// `count` parameter is in raw bytes.
            ///
            /// *See also: [`ptr::add`][ptr_add]*
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null, or take it out of bounds for its allocation.
            /// - The value pointed by the resulting pointer must outlive the lifetime of this pointer.
            ///
            /// [ptr_add]: https://doc.rust-lang.org/std/primitive.pointer.html#method.add
            #[inline]
            pub unsafe fn byte_add(self, count: usize) -> Self {
                Self(
                    // SAFETY: The caller upholds safety for `add` and ensures the result is not null.
                    unsafe { NonNull::new_unchecked(self.as_ptr().add(count)) },
                    PhantomData,
                )
            }
        }
    };
}

impl_ptr!(Ptr);
impl_ptr!(PtrMut);
impl_ptr!(OwnPtr);

impl<'a> Ptr<'a> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value and be propperly aligned for the errased type.
    /// - The lifetime `'a` must be constrained such that this `Ptr` will stay valid and nothing
    ///   can mutate the pointee while this `Ptr` is live except through inner mutability ([`UnsafeCell`]).
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Transforms this `Ptr` into an [`PtrMut`].
    ///
    /// # Safety
    /// - The data pointed to by this `Ptr` must be valid for writes.
    /// - There must be no active references (mutable or otherwise) to the data underlying this `Ptr`.
    /// - Another [`PtrMut`] for the same `Ptr` must not be created until the first is dropped.
    #[inline]
    pub unsafe fn unique(inner: NonNull<u8>) -> PtrMut<'a> {
        PtrMut(inner, PhantomData)
    }

    /// Transforms this `Ptr<T>` into a `&T` with the same lifetime.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this `Ptr`.
    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: The caller ensures the pointee is of type `T` and the pointer can be dereferenced.
        unsafe { &*ptr }
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<'a> {
    #[inline]
    fn from(val: &'a T) -> Self {
        // SAFETY: The returned pointer has the same lifetime as the passed reference.
        // Access is immutable.
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a> PtrMut<'a> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value and be propperly aligned for the errased type.
    /// - The lifetime `'a` must be constrained such that this `PtrMut` will stay valid and nothing
    ///   else can read or mutate the pointee while this `PtrMut` is live.
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets an new immutable reference from this mutable reference.
    #[inline]
    pub fn as_ref(&self) -> Ptr<'_> {
        // SAFETY: The `PtrMut` type's guarantees about the validity of this pointer are a superset of `Ptr`s guarantees
        unsafe { Ptr::new(self.0) }
    }

    /// Transforms this `PtrMut` into an [`OwnPtr`].
    ///
    /// # Safety
    /// Must have right to drop or move out of `PtrMut`.
    #[inline]
    pub unsafe fn promote(self) -> OwnPtr<'a> {
        OwnPtr(self.0, PhantomData)
    }

    /// Transforms this `PtrMut<T>` into a `&mut T` with the same lifetime
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`PtrMut`].
    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: The caller ensures the pointee is of type `T` and the pointer can be dereferenced.
        unsafe { &mut *ptr }
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        // SAFETY: The returned pointer has the same lifetime as the passed reference.
        // Access is immutable.
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a> OwnPtr<'a> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value and be propperly aligned for the errased type.
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// This exists mostly to reduce compile times;
    /// code is only duplicated per type, rather than per function called.
    ///
    /// # Safety
    /// Safety constraints of [`PtrMut::promote`] must be upheld.
    unsafe fn from_internal<T>(temp: &mut ManuallyDrop<T>) -> OwnPtr<'_> {
        // SAFETY: The constraints of `promote` are upheld by caller.
        unsafe { PtrMut::from(&mut *temp).promote() }
    }

    /// Consumes a value and creates an [`OwningPtr`] to it while ensuring a double drop does not happen.
    #[inline]
    pub fn from<T, R, F: FnOnce(OwnPtr<'_>) -> R>(value: T, func: F) -> R {
        let mut val = ManuallyDrop::new(value);
        // SAFETY: The value behind the pointer will not get dropped or observed later,
        // so it's safe to promote it to an owning pointer.
        func(unsafe { Self::from_internal(&mut val) })
    }

    /// Consumes the `OwnPtr` to obtain ownership of the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased type for this `OwnPtr`.
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: The caller ensure the pointee is of type `T` and uphold safety for `read`.
        unsafe { ptr.read() }
    }

    /// Consumes the `OwnPtr`]to drop the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased type for this `OwnPtr`.
    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: The caller ensure the pointee is of type `T` and uphold safety for `drop_in_place`.
        unsafe {
            ptr.drop_in_place();
        }
    }

    /// Gets an new immutable reference from this mutable reference.
    #[inline]
    pub fn as_ref(&self) -> Ptr<'_> {
        // SAFETY: The `PtrMut` type's guarantees about the validity of this pointer are a superset of `Ptr`s guarantees
        unsafe { Ptr::new(self.0) }
    }

    /// Gets an new immutable reference from this mutable reference.
    #[inline]
    pub fn as_mut(&self) -> PtrMut<'_> {
        // SAFETY: The `PtrMut` type's guarantees about the validity of this pointer are a superset of `Ptr`s guarantees
        unsafe { PtrMut::new(self.0) }
    }
}

pub unsafe fn drop_in_place<T>(ptr: OwnPtr<'_>) {
    unsafe {
        ptr.drop_as::<T>();
    }
}

/// Returns the [`Drop`] implementation for the type `T`.
#[inline]
pub const fn get_drop_fn<T>() -> Option<DropFn> {
    if core::mem::needs_drop::<T>() {
        Some(crate::drop_in_place::<T> as _)
    } else {
        None
    }
}
