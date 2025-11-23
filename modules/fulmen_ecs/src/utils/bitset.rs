use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[cfg(target_pointer_width = "64")]
type BitBlock = u64;
#[cfg(not(target_pointer_width = "64"))]
type BitBlock = u32;

#[derive(Default, Clone)]
pub struct BitSetMut {
    bits: Vec<BitBlock>,
}
#[derive(Default, Clone)]
pub struct BitSet {
    bits: Box<[BitBlock]>,
}

// region: Common BitSet Implementation code

macro_rules! impl_bitset {
    ($ty:ty) => {
        impl $ty {
            /// Calculates the index of the [`BitBlock`] that corresponds to the nth `bit`.
            #[inline(always)]
            const fn block_index(bit: usize) -> usize {
                bit / BitBlock::BITS as usize
            }

            /// Calculates the index inside the [`BitBlock`] that corresponds to the nth `bit`.
            #[inline(always)]
            const fn bit_index(bit: usize) -> usize {
                bit % BitBlock::BITS as usize
            }

            /// Returns `true` if the set has no [`BitBlocks`](`BitBlock`) allocated.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.bits.is_empty()
            }

            /// Returns the number of [`BitBlocks`](`BitBlock`) in the set.
            #[inline]
            pub fn block_count(&self) -> usize {
                self.bits.len()
            }

            /// Returns a reference to the [`BitBlock`] with the nth `bit`.
            #[inline(always)]
            fn get_block(&self, bit: usize) -> Option<&BitBlock> {
                self.bits.get(Self::block_index(bit))
            }

            /// Checks wether or not this `BitSetMut` contains the nth `bit`
            pub fn contains<'a, 'b>(&'a self, bit: usize) -> bool {
                if let Some(block) = self.get_block(bit) {
                    let bit = 1 << Self::bit_index(bit);
                    return (*block & bit) == bit;
                } else {
                    return false;
                }
            }

            /// Checks wether or not this `BitSetMut` contains all the bits on the `other` `BitSetMut`
            pub fn contains_all<'a, 'b>(&'a self, other: &'b Self) -> bool {
                self & other == *other
            }
        }

        impl PartialEq<$ty> for $ty {
            fn eq(&self, other: &$ty) -> bool {
                self.bits == other.bits
            }
        }

        impl Eq for $ty {}

        impl PartialOrd for $ty {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $ty {
            fn cmp(&self, other: &Self) -> Ordering {
                let max_len = usize::max(self.bits.len(), other.bits.len());

                // Compare from least to most significant `BitBlock`
                for i in 0..max_len {
                    let a = self.bits.get(i).unwrap_or(&0);
                    let b = other.bits.get(i).unwrap_or(&0);

                    match a.cmp(b) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }

                // If no different bits were found
                Ordering::Equal
            }
        }
    };
}

impl_bitset!(BitSetMut);
impl_bitset!(BitSet);

// endregion

impl BitSetMut {
    /// Creates a new, empty `BitSetMut`.
    #[inline]
    pub const fn new() -> Self {
        BitSetMut { bits: Vec::new() }
    }

    /// Creates a new `BitSetMut` from a raw slice of `bits`.
    #[inline]
    pub fn from_raw_bits(bits: &[BitBlock]) -> Self {
        let mut set = BitSetMut {
            bits: bits.to_vec(),
        };
        set.trim_trailing_zeros();
        set
    }

    /// Bit manipulation code for [`Self::add`] and [`Self::add_all`].
    #[inline(always)]
    const fn set_bit(block: &mut BitBlock, bit: usize) {
        *block |= 1 << Self::bit_index(bit);
    }

    /// Bit manipulation code for [`Self::remove`] and [`Self::remove_all`].
    #[inline(always)]
    const fn unset_bit(block: &mut BitBlock, bit: usize) {
        *block &= !(1 << Self::bit_index(bit));
    }

    /// Returns a mutable reference to the [`BitBlock`] with the nth `bit`.
    #[inline(always)]
    fn get_block_mut(&mut self, bit: usize) -> Option<&mut BitBlock> {
        self.bits.get_mut(Self::block_index(bit))
    }

    /// Returns a mutable reference to the [`BitBlock`] with the nth `bit`,
    /// inserting new ones if not already present.
    #[inline]
    fn get_or_insert_block(&mut self, bit: usize) -> &mut BitBlock {
        // If the block is not present, insert it and all missing blocks in between.
        if let None = self.get_block(bit) {
            let block_index = Self::block_index(bit);
            if block_index >= self.bits.len() {
                self.bits.resize_with(block_index + 1, || 0);
            }
        }

        self.get_block_mut(bit).unwrap()
    }

    /// Adds the nth `bit` to the set.
    #[inline]
    pub fn add(&mut self, bit: usize) {
        let block = self.get_or_insert_block(bit);
        Self::set_bit(block, bit);
    }

    /// Adds several nth `bits` to the set.
    #[inline]
    pub fn add_all(&mut self, bits: &[usize]) {
        bits.iter().for_each(|bit| self.add(*bit));
    }

    /// Removes the nth `bit` from the set.
    #[inline]
    pub fn remove(&mut self, bit: usize) {
        if let Some(block) = self.get_block_mut(bit) {
            Self::unset_bit(block, bit);
            self.trim_trailing_zeros();
        }
    }

    /// Removes several nth `bits` from the set.
    #[inline]
    pub fn remove_all(&mut self, bits: &[usize]) {
        bits.iter().for_each(|bit| {
            if let Some(block) = self.get_block_mut(*bit) {
                Self::unset_bit(block, *bit);
            }
        });
        self.trim_trailing_zeros();
    }

    /// Trims all the [`BitBlocks`](`BitBlock`) at the end of the `BitSetMut` that contain no values.
    fn trim_trailing_zeros(&mut self) {
        while let Some(&0) = self.bits.last() {
            self.bits.pop();
        }
    }
}

impl BitSet {
    /// Creates a new `BitSet` from a raw slice of `bits`.
    #[inline]
    pub fn from_raw_bits(bits: &[BitBlock]) -> Self {
        BitSetMut::from_raw_bits(bits).into()
    }
}

impl From<BitSetMut> for BitSet {
    fn from(set: BitSetMut) -> Self {
        BitSet {
            bits: set.bits.into_boxed_slice(),
        }
    }
}

// region: Bitwise Operation Implementations

macro_rules! gen_bit_trucation {
    // Conditional truncation
    (T $self: ident, $common_bits: ident) => {};
    (T {()} $self: ident, $common_bits: ident) => {};
    (T [()] $self: ident, $common_bits: ident) => {
        // Force the move
        let mut new_bits = core::mem::take(&mut $self.bits).into_vec();
        new_bits.truncate($common_bits);

        // Move data back
        $self.bits = new_bits.into_boxed_slice();
    };
    (T {()} [()] $self: ident, $common_bits: ident) => {
        $self.bits.truncate($common_bits);
    };
}

macro_rules! gen_mut_binop_impls {
    // Main macro body
    ($type:ty [$op:tt $op2:tt] [$($mut_bits:tt)?] [$($truncate:tt)?] [$trait: ident]($trait_fn_name: ident) [$trait2: ident]($trait2_fn_name: ident)) => {
        // A op &B
        impl $trait<&$type> for $type {
            type Output = $type;

            fn $trait_fn_name(mut self, rhs: &$type) -> Self::Output {
                let common_bits = usize::min(self.bits.len(), rhs.bits.len());
                self.bits[..common_bits]
                    .iter_mut()
                    .zip(rhs.bits[..common_bits].iter())
                    .for_each(|(a, b)| *a $op2 *b);

                gen_bit_trucation!(T $({$mut_bits})? $([$truncate])? self, common_bits);
                self
            }
        }

        // A op &mut B -> A op &B
        impl $trait<&mut $type> for $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &mut $type) -> Self::Output {
                self $op rhs as &$type // Reuse impl
            }
        }

        // A op B -> A op &B
        impl $trait<$type> for $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: $type) -> Self::Output {
                self $op &rhs // Reuse impl
            }
        }

        // &A op &B -> Clone A op &B
        impl $trait<&$type> for &$type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &$type) -> Self::Output {
                self.clone() $op rhs // Reuse impl
            }
        }

        // &A op &mut B -> &A op &B
        impl $trait<&mut $type> for &$type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &mut $type) -> Self::Output {
                self $op rhs as &$type // Reuse impl
            }
        }

        // &mut A op &mut B -> &A op &B
        impl $trait<&mut $type> for &mut $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &mut $type) -> Self::Output {
                self as &$type $op rhs as &$type // Reuse impl
            }
        }

        // &mut A op &B -> &A op &B
        impl $trait<&$type> for &mut $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &$type) -> Self::Output {
                self as &$type $op rhs // Reuse impl
            }
        }

        // &mut T op2= T -> &mut T op2= &T
        impl $trait2<$type> for $type {
            #[inline]
            fn $trait2_fn_name(&mut self, rhs: $type) {
                *self $op2 &rhs; // Reuse impl
            }
        }

        // &mut T op2= &mut T -> &mut T op2= &T
        impl $trait2<&mut $type> for $type {
            #[inline]
            fn $trait2_fn_name(&mut self, rhs: &mut $type) {
                *self $op2 rhs as &$type; // Reuse impl
            }
        }

        // &mut T op2= &T -> A
        impl $trait2<&$type> for $type {
            fn $trait2_fn_name(&mut self, rhs: &$type) {
                let common_bits = usize::min(self.bits.len(), rhs.bits.len());
                self.bits[..common_bits]
                    .iter_mut()
                    .zip(rhs.bits[..common_bits].iter())
                    .for_each(|(a, b)| *a $op2 *b);

                gen_bit_trucation!(T $({$mut_bits})? $([$truncate])? self, common_bits);
            }
        }
    };
}

macro_rules! gen_binop_impls {
    // Main macro body
    ($type:ty [$op:tt] [$($mut_bits:tt)?] [$($truncate:tt)?] [$trait: ident]($trait_fn_name: ident)) => {
        // &A op &B -> Clone A op &B
        impl $trait<&$type> for &$type {
            type Output = $type;

            fn $trait_fn_name(self, rhs: &$type) -> Self::Output {
                let mut new_self = self.clone();
                let common_bits = usize::min(new_self.bits.len(), rhs.bits.len());
                new_self.bits[..common_bits]
                    .iter_mut()
                    .zip(rhs.bits[..common_bits].iter())
                    .for_each(|(a, b)| *a = *a $op *b);

                gen_bit_trucation!(T $({$mut_bits})? $([$truncate])? new_self, common_bits);
                new_self
            }
        }

        // &A op &mut B -> &A op &B
        impl $trait<&mut $type> for &$type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &mut $type) -> Self::Output {
                self $op rhs as &$type // Reuse impl
            }
        }

        // &mut A op &mut B -> &A op &B
        impl $trait<&mut $type> for &mut $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &mut $type) -> Self::Output {
                self as &$type $op rhs as &$type // Reuse impl
            }
        }

        // &mut A op &B -> &A op &B
        impl $trait<&$type> for &mut $type {
            type Output = $type;

            #[inline]
            fn $trait_fn_name(self, rhs: &$type) -> Self::Output {
                self as &$type $op rhs // Reuse impl
            }
        }
    };
}

gen_mut_binop_impls!(BitSetMut [& &=] [()] [()] [BitAnd](bitand) [BitAndAssign](bitand_assign));
gen_mut_binop_impls!(BitSetMut [| |=] [()] [  ] [BitOr](bitor)   [BitOrAssign](bitor_assign));
gen_mut_binop_impls!(BitSetMut [^ ^=] [()] [  ] [BitXor](bitxor) [BitXorAssign](bitxor_assign));

impl Not for BitSetMut {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        for block in self.bits.iter_mut() {
            *block = !*block;
        }
        self
    }
}

gen_binop_impls!(BitSet [&] [  ] [()] [BitAnd](bitand));
gen_binop_impls!(BitSet [|] [  ] [  ] [BitOr](bitor));
gen_binop_impls!(BitSet [^] [  ] [  ] [BitXor](bitxor));

impl Not for BitSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        let mut new_self = self.clone();
        // Force the move
        let mut new_bits = core::mem::take(&mut new_self.bits).into_vec();
        for block in new_bits.iter_mut() {
            *block = !*block;
        }

        // Move data back
        new_self.bits = new_bits.into_boxed_slice();
        new_self
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::{BitBlock, BitSet, BitSetMut};
    use std::cmp::Ordering;

    const LHS: BitBlock = 0b10000011;
    const RHS: BitBlock = 0b10010001;

    macro_rules! assert_bitwise_op {
        ([VALUE] [$op:tt] <- $source:ident | ($lhs:tt)($rhs:tt)) => {
            // By Value and Value: both get consumed
            let (lhs, rhs) = $source();
            let result = lhs $op rhs;
            assert_eq!($lhs $op $rhs, result.bits[0]);

            // By Value and Ref: only lhs get consumed
            let (lhs, rhs) = $source();
            let result = lhs $op (&rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);

            // By Value and Mut : only lhs get consumed
            let (lhs, mut rhs) = $source();
            let result = lhs $op (&mut rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);
        };
        ([REFS] [$op:tt] <- $source:ident | ($lhs:tt)($rhs:tt)) => {
            // By Ref : new value generated/allocated
            let (lhs, rhs) = $source();
            let result = (&lhs) $op (&rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);

            // By Ref and Mut : new value generated/allocated
            let (lhs, mut rhs) = $source();
            let result = (&lhs) $op (&mut rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);

            // By Mut and Ref : new value generated/allocated
            let (mut lhs, rhs) = $source();
            let result = (&mut lhs) $op (&rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);

            // By Mut and Mut : new value generated/allocated
            let (mut lhs, mut rhs) = $source();
            let result = (&mut lhs) $op (&mut rhs);
            assert_eq!($lhs $op $rhs, result.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);
        };
        ([ASSIGN] [$op:tt $op2:tt] <- $source:ident | ($lhs:tt)($rhs:tt)) => {
            // By Value : rhs consumed
            let (mut lhs, rhs) = $source();
            lhs $op2 rhs;
            assert_eq!($lhs $op $rhs, lhs.bits[0]);

            // By REF : No Values consumed
            let (mut lhs, rhs) = $source();
            lhs $op2 &rhs;
            assert_eq!($lhs $op $rhs, lhs.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);

            // By MUT : No Values consumed
            let (mut lhs, mut rhs) = $source();
            lhs $op2 &mut rhs;
            assert_eq!($lhs $op $rhs, lhs.bits[0]);
            assert_eq!($rhs, rhs.bits[0]);
        };
    }

    macro_rules! test_eq_and_ord_invariants {
        ($type:ty) => {
            #[test]
            fn test_eq_reflexive_invariant() {
                let a = <$type>::from_raw_bits(&[0b00000001]);

                assert!(a == a);
            }

            #[test]
            fn test_eq_symetric_invariant() {
                let a = <$type>::from_raw_bits(&[0b00000001]);
                let b = <$type>::from_raw_bits(&[0b00000001]);

                assert!(a == b && b == a);
            }

            #[test]
            fn test_eq_transitive_invariant() {
                let a = <$type>::from_raw_bits(&[0b00000001]);
                let b = <$type>::from_raw_bits(&[0b00000001]);
                let c = <$type>::from_raw_bits(&[0b00000001]);

                assert!(a == b && b == c && c == a);
            }

            #[test]
            fn test_ord_antisymetric_invariant() {
                let a = <$type>::from_raw_bits(&[0b00000001]);
                let b = <$type>::from_raw_bits(&[0b00000001]);
                let c = <$type>::from_raw_bits(&[0b00000010]);

                assert!(a <= b && b <= a);
                assert!(a == b);

                assert!(a <= c && !(c <= a));
                assert!(a != c);
            }

            #[test]
            fn test_ord_transitive_invariant() {
                let a = <$type>::from_raw_bits(&[0b00000001]);
                let b = <$type>::from_raw_bits(&[0b00000010]);
                let c = <$type>::from_raw_bits(&[0b00000100]);

                assert!(a < b && b < c);
                assert!(a < c);
            }

            #[test]
            fn test_ord_eq_consistency() {
                let a = <$type>::from_raw_bits(&[0b00000001]);
                let b = <$type>::from_raw_bits(&[0b00000001]);
                let c = <$type>::from_raw_bits(&[0b00000010]);

                assert!(a == b);
                assert!(a.cmp(&b) == Ordering::Equal);

                assert!(a != c);
                assert!(a < c);
                assert!(a.cmp(&c) == Ordering::Less)
            }
        };
    }

    mod bitset_mut {
        use super::BitSetMut;
        use super::{BitBlock, LHS, Ordering, RHS};

        #[test]
        fn new() {
            let set = BitSetMut::new();
            assert_eq!(true, set.is_empty());
        }

        #[test]
        fn from_raw_bits() {
            let set = BitSetMut::from_raw_bits(&[0b01111001, 0b00000001]);
            assert_eq!(false, set.is_empty());

            // Block 0
            assert_eq!(true, set.contains(0));
            assert_eq!(true, set.contains(3));
            assert_eq!(true, set.contains(6));
            assert_eq!(false, set.contains(1));

            // Block 1
            assert_eq!(true, set.contains(BitBlock::BITS as usize));
            assert_eq!(false, set.contains(BitBlock::BITS as usize + 1));
        }

        #[test]
        fn add() {
            let mut set = BitSetMut::from_raw_bits(&[0b00000001]);
            assert_eq!(1, set.block_count());

            // Set the nth bit corresponding to the first bit of the second block, to grow the set.
            set.add(BitBlock::BITS as usize);
            assert_eq!(2, set.block_count());
        }

        #[test]
        fn add_all() {
            let mut set = BitSetMut::new();
            set.add_all(&[BitBlock::BITS as usize, 4, 0]);

            assert_eq!(2, set.block_count());
        }

        #[test]
        fn remove() {
            let mut set = BitSetMut::from_raw_bits(&[0b10000001]);
            assert_eq!(true, set.contains(0));

            set.remove(0);
            assert_eq!(false, set.contains(0));
        }

        #[test]
        fn remove_all() {
            let mut set = BitSetMut::from_raw_bits(&[0b10000001, 0b000000001]);
            assert_eq!(2, set.block_count());

            set.remove_all(&[0, BitBlock::BITS as usize]);
            assert_eq!(1, set.block_count());
            assert_eq!(false, set.contains(0));
            assert_eq!(false, set.contains(BitBlock::BITS as usize));
        }

        #[test]
        fn trim_trailing_zeros() {
            let mut set = BitSetMut::from_raw_bits(&[0b00000001, 0b00000001, 0, 0, 0b00000001]);
            assert_eq!(5, set.block_count());

            set.trim_trailing_zeros();
            assert_eq!(5, set.block_count());

            // Manually unset bit
            let block = set.get_block_mut((BitBlock::BITS * 4) as usize).unwrap();
            BitSetMut::unset_bit(block, (BitBlock::BITS * 4) as usize);
            assert_eq!(5, set.block_count());

            set.trim_trailing_zeros();
            assert_eq!(2, set.block_count());
        }

        #[test]
        fn contains() {
            let set = BitSetMut::from_raw_bits(&[0b10011001, 0b000000001]);

            assert_eq!(false, set.contains(1));
            assert_eq!(true, set.contains(0));
            assert_eq!(true, set.contains(BitBlock::BITS as usize));
        }

        #[test]
        fn contains_all() {
            let a = BitSetMut::from_raw_bits(&[0b10011001, 0b000000001]);
            let b = BitSetMut::from_raw_bits(&[0b10011000]);

            assert_eq!(true, a.contains_all(&b))
        }

        test_eq_and_ord_invariants!(BitSetMut);

        fn get_lhs_rhs() -> (BitSetMut, BitSetMut) {
            (<BitSetMut>::from_raw_bits(&[LHS]), <BitSetMut>::from_raw_bits(&[RHS]))
        }

        #[test]
        fn bitwise_not() {
            let mut set = BitSetMut::from_raw_bits(&[LHS]);
            set = !set;

            assert_eq!(BitBlock::MAX ^ LHS, set.bits[0]);
        }

        #[test]
        fn bitwise_and() {
            assert_bitwise_op!([VALUE] [&] <- get_lhs_rhs | (LHS)(RHS));
            assert_bitwise_op!([REFS]  [&] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_and_assign() {
            assert_bitwise_op!([ASSIGN] [& &=] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_or() {
            assert_bitwise_op!([VALUE] [|] <- get_lhs_rhs | (LHS)(RHS));
            assert_bitwise_op!([REFS]  [|] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_or_assign() {
            assert_bitwise_op!([ASSIGN] [| |=] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_xor() {
            assert_bitwise_op!([VALUE] [^] <- get_lhs_rhs | (LHS)(RHS));
            assert_bitwise_op!([REFS]  [^] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_xor_assign() {
            assert_bitwise_op!([ASSIGN] [^ ^=] <- get_lhs_rhs | (LHS)(RHS));
        }
    }

    mod bitset {
        use super::BitSet;
        use super::{BitBlock, LHS, Ordering, RHS};

        #[test]
        fn from_raw_bits() {
            let set = BitSet::from_raw_bits(&[0b01111001, 0b00000001]);
            assert_eq!(false, set.is_empty());

            // Block 0
            assert_eq!(true, set.contains(0));
            assert_eq!(true, set.contains(3));
            assert_eq!(true, set.contains(6));
            assert_eq!(false, set.contains(1));

            // Block 1
            assert_eq!(true, set.contains(BitBlock::BITS as usize));
            assert_eq!(false, set.contains(BitBlock::BITS as usize + 1));
        }

        #[test]
        fn contains() {
            let set = BitSet::from_raw_bits(&[0b10011001, 0b000000001]);

            assert_eq!(false, set.contains(1));
            assert_eq!(true, set.contains(0));
            assert_eq!(true, set.contains(BitBlock::BITS as usize));
        }

        #[test]
        fn contains_all() {
            let a = BitSet::from_raw_bits(&[0b10011001, 0b000000001]);
            let b = BitSet::from_raw_bits(&[0b10011000]);

            assert_eq!(true, a.contains_all(&b))
        }

        test_eq_and_ord_invariants!(BitSet);

        fn get_lhs_rhs() -> (BitSet, BitSet) {
            (BitSet::from_raw_bits(&[LHS]), BitSet::from_raw_bits(&[RHS]))
        }

        #[test]
        fn bitwise_not() {
            let mut set = BitSet::from_raw_bits(&[LHS]);
            set = !set;

            assert_eq!(BitBlock::MAX ^ LHS, set.bits[0]);
        }

        #[test]
        fn bitwise_and() {
            assert_bitwise_op!([REFS] [&] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_or() {
            assert_bitwise_op!([REFS] [|] <- get_lhs_rhs | (LHS)(RHS));
        }

        #[test]
        fn bitwise_xor() {
            assert_bitwise_op!([REFS] [^] <- get_lhs_rhs | (LHS)(RHS));
        }
    }
}
