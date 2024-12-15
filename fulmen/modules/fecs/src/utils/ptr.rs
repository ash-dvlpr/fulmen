use std::{marker, ptr::NonNull};

#[repr(transparent)]
pub struct RawPtr<'a>(NonNull<u8>, marker::PhantomData<&'a u8>);

/// Raw Box-like pointer without type information
///
/// Act's as a Box 
#[repr(transparent)]
pub struct RawMutPtr<'a>(NonNull<u8>, marker::PhantomData<&'a mut u8>);
