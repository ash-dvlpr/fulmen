use core::hash::Hash;
use core::marker::PhantomData;

pub trait SparseSetIndex: Clone + PartialEq + Eq + Hash {
    /// Gets the sparse set index of this value.
    fn sparse_set_index(&self) -> usize;

    /// Creates a new `impl SparseSetIndex` from the specified index.
    fn get_sparse_set_index(value: usize) -> Self;
}

macro_rules! impl_sparse_set_index {
    ($($ty:ty),+) => {
        $(impl SparseSetIndex for $ty {
            #[inline]
            fn sparse_set_index(&self) -> usize {
                *self as usize
            }

            #[inline]
            fn get_sparse_set_index(value: usize) -> Self {
                value as $ty
            }
        })*
    };
}

impl_sparse_set_index!(u8, u16, u32, u64, usize);

/// Alias for `usize` used to clarify that the value is used to index `dense`
type DenseIndex = usize;

/// A data structure that densly stores values while being accesible by sparse indices.
///
/// `I` is the index type, and `V` is the type of the stored values.
pub struct SparseSet<I: SparseSetIndex, V: 'static> {
    // Buffer where the data gets densly stored.
    dense: Vec<V>,
    /// Cached indices used when moving values inside of the dense list.
    indices: Vec<I>,
    /// Mapping between indices and the actual index inside of `dense`.
    sparse: SparseArray<I, DenseIndex>,
}

impl<I: SparseSetIndex, V> SparseSet<I, V> {
    /// Constructs a new, empty `SparseSet` for the specified type.
    #[inline]
    pub const fn new() -> Self {
        Self {
            dense: Vec::new(),
            indices: Vec::new(),
            sparse: SparseArray::new(),
        }
    }

    /// Creates a new `SparseSet` with a specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dense: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            sparse: SparseArray::new(),
        }
    }

    /// Returns the number of elements stored in the sparse set.
    #[inline]
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns the total number of elements the `SparseSet` can hold without needing to reallocate.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.dense.capacity()
    }

    /// Returns `true` if the sparse set contains a value for `index`.
    #[inline]
    pub fn contains(&self, index: I) -> bool {
        self.sparse.contains(index)
    }

    /// Returns `true` if the sparse set contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense.len() == 0
    }

    /// Returns a reference to the value for `index`.
    ///
    /// Returns `None` if `index` does not have a value in the sparse set.
    pub fn get(&self, index: I) -> Option<&V> {
        self.sparse.get(index).map(|dense_index| {
            // SAFETY: if the sparse index points to something in the dense vec, it exists
            unsafe { self.dense.get_unchecked(*dense_index) }
        })
    }

    /// Returns a mutable reference to the value for `index`.
    ///
    /// Returns `None` if `index` does not have a value in the sparse set.
    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        let dense = &mut self.dense;
        self.sparse.get(index).map(move |dense_index| {
            // SAFETY: if the sparse index points to something in the dense vec, it exists
            unsafe { dense.get_unchecked_mut(*dense_index) }
        })
    }

    /// Returns a reference to the value for `index`, inserting one computed from `func`
    /// if not already present.
    pub fn get_or_insert_with(&mut self, index: I, func: impl FnOnce() -> V) -> &mut V {
        if let Some(dense_index) = self.sparse.get(index.clone()).cloned() {
            // SAFETY: dense indices stored in self.sparse always exist
            unsafe { self.dense.get_unchecked_mut(dense_index) }
        } else {
            let value = func();

            let dense_index = self.dense.len();
            self.sparse.insert(index.clone(), dense_index);
            self.indices.push(index);
            self.dense.push(value);

            // SAFETY: dense index was just populated above
            unsafe { self.dense.get_unchecked_mut(dense_index) }
        }
    }

    /// Inserts `value` at `index`.
    ///
    /// If a value was already present at `index`, it will be overwritten.
    pub fn insert(&mut self, index: I, value: V) {
        if let Some(dense_index) = self.sparse.get(index.clone()).cloned() {
            // SAFETY: dense indices stored in self.sparse always exist
            unsafe { *self.dense.get_unchecked_mut(dense_index) = value };
        } else {
            self.sparse.insert(index.clone(), self.dense.len());
            self.indices.push(index);
            self.dense.push(value);
        }
    }

    /// Removes and returns the value for `index`.
    ///
    /// Returns `None` if `index` does not have a value in the sparse set.
    pub fn remove(&mut self, index: I) -> Option<V> {
        self.sparse.remove(index).map(|dense_index| {
            // Check wherether we have to update the indices in the dense set
            let is_last = dense_index == self.dense.len() - 1;

            // Remove the cached index and the value from the sparse set
            let value = self.dense.swap_remove(dense_index);
            self.indices.swap_remove(dense_index);

            // If values were moved while removing, update the indices in the sparse set
            if !is_last {
                let swapped_index = self.indices[dense_index].clone();
                *self.sparse.get_mut(swapped_index).unwrap() = dense_index;
            }

            value
        })
    }

    /// Returns an iterator visiting all keys (indices) in arbitrary order.
    pub fn indices(&self) -> impl Iterator<Item = I> + Clone + '_ {
        self.indices.iter().cloned()
    }

    /// Returns an iterator visiting all values in arbitrary order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.dense.iter()
    }

    /// Returns an iterator visiting all values mutably in arbitrary order.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.dense.iter_mut()
    }

    /// Clears all of the elements from the sparse set.
    pub fn clear(&mut self) {
        self.dense.clear();
        self.indices.clear();
        self.sparse.clear();
    }
}

#[derive(Debug)]
pub(crate) struct SparseArray<I: SparseSetIndex, V> {
    values: Vec<Option<V>>,
    marker: PhantomData<I>,
}

impl<I: SparseSetIndex, V> SparseArray<I, V> {
    /// Constructs a new, empty `SparseArray` for the specified types
    #[inline]
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Returns `true` if the collection contains a value for the specified `index`.
    #[inline]
    pub fn contains(&self, index: I) -> bool {
        let index = index.sparse_set_index();
        self.values.get(index).is_some_and(Option::is_some)
    }

    /// Returns a reference to the value at `index`.
    ///
    /// Returns `None` if `index` does not have a value or if `index` is out of bounds.
    #[inline]
    pub fn get(&self, index: I) -> Option<&V> {
        let index = index.sparse_set_index();
        self.values.get(index).and_then(Option::as_ref)
    }

    /// Inserts `value` at `index` in the array.
    ///
    /// If `index` is out-of-bounds, this will enlarge the buffer to accommodate it.
    #[inline]
    pub fn insert(&mut self, index: I, value: V) {
        let index = index.sparse_set_index();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    /// Returns a mutable reference to the value at `index`.
    ///
    /// Returns `None` if `index` does not have a value or if `index` is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        let index = index.sparse_set_index();
        self.values.get_mut(index).and_then(Option::as_mut)
    }

    /// Removes and returns the value stored at `index`.
    ///
    /// Returns `None` if `index` did not have a value or if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: I) -> Option<V> {
        let index = index.sparse_set_index();
        self.values.get_mut(index).and_then(Option::take)
    }

    /// Removes all of the values stored within.
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::SparseSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    struct TestStruct(pub u32);

    type SparseID = usize;
    type TestSparseSet = SparseSet<SparseID, TestStruct>;

    #[test]
    fn new_sparse_set() {
        let set = TestSparseSet::new();
        assert!(set.is_empty());
    }

    #[test]
    fn sparse_set_empty() {
        let mut set = TestSparseSet::new();
        assert!(set.is_empty());

        set.insert(2, TestStruct(2));
        assert!(!set.is_empty());

        set.clear();
        assert!(set.is_empty());
    }

    fn sum_all(set: &TestSparseSet) -> Option<u32> {
        set.values().map(|v| v.0).reduce(|acc, v| acc + v)
    }

    fn as_slice(set: &TestSparseSet) -> &[u32] {
        let slice = &set.dense[..];

        // SAFETY: TestStruct is #[repr(transparent)] over u32, so layouts match
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u32, slice.len()) }
    }

    #[test]
    fn sparse_set_contains() {
        let mut set = TestSparseSet::new();
        assert!(!set.contains(2));

        set.insert(2, TestStruct(2));
        assert!(set.contains(2));
    }

    #[test]
    fn sparse_set_internals() {
        let mut set = TestSparseSet::new();

        // Empty
        assert_eq!(None, sum_all(&set));
        assert_eq!(0, set.dense.len());
        assert_eq!(0, set.indices.len());
        assert_eq!(0, set.sparse.values.len());

        // After some insertions
        set.insert(4, TestStruct(4));
        set.insert(0, TestStruct(0));
        set.insert(1, TestStruct(1));

        assert_eq!(Some(&2), set.sparse.get(1)); // Check value's index inside `sparse`

        assert_eq!(Some(5), sum_all(&set));
        assert_eq!(5, set.sparse.values.len());
        assert_eq!([4, 0, 1], as_slice(&set));
        assert_eq!([4, 0, 1], set.indices[..]);

        // After some removals
        assert_eq!(None, set.remove(10));
        assert_eq!(Some(TestStruct(4)), set.remove(4));

        assert_eq!(Some(&0), set.sparse.get(1)); // Check value's index inside `sparse`

        assert_eq!(Some(1), sum_all(&set));
        assert_eq!(5, set.sparse.values.len()); // Sparse keeps all the None values

        // Check that the last value is now at the front
        assert_eq!([1, 0], as_slice(&set));
        assert_eq!([1, 0], set.indices[..]);
    }
}
