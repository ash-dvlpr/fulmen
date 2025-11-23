use core::hash::{BuildHasher, Hasher};

/// [`BuildHasher`] for types that already contain a high-quality hash.
#[derive(Clone, Default)]
pub struct NoOpHash;

impl BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher::default()
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct NoOpHasher(u64);

// This for types that act like hashes, to avoid re-hashing that value.
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = bytes
            .iter()
            .fold(self.0, |hash, b| hash.rotate_left(8).wrapping_add(*b as u64));
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
