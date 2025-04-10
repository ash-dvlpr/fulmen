use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[cfg(target_pointer_width = "64")]
type BitBlock = u64;
#[cfg(not(target_pointer_width = "64"))]
type BitBlock = u32;

#[derive(Default, Clone)]
// pub struct BitSet {
pub struct BitSet {
    bits: Vec<BitBlock>,
}

impl BitSet {
    /// Creates a new, empty `BitSet`.
    #[inline]
    pub const fn new() -> BitSet {
        BitSet { bits: Vec::new() }
    }

    /// Creates a new `BitSet` from a raw slice of `bits`.
    #[inline]
    pub fn from_raw_bits(bits: &[BitBlock]) -> BitSet {
        BitSet {
            bits: bits.to_vec(),
        }
    }

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
        *block |= 1 << Self::bit_index(bit);
    }

    /// Adds several nth `bits` to the set.
    #[inline]
    pub fn add_all(&mut self, bits: &[usize]) {
        bits.iter().for_each(|bit| self.add(*bit));
    }

    /// Removes the nth `bit` to the set.
    #[inline(always)]
    fn remove_internal(&mut self, bit: usize) {
        let block = self.get_or_insert_block(bit);
        *block &= !(1 << Self::bit_index(bit));
    }

    /// Removes the nth `bit` to the set.
    #[inline]
    pub fn remove(&mut self, bit: usize) {
        self.remove_internal(bit);
    }

    /// Adds several nth `bits` to the set.
    #[inline]
    pub fn remove_all(&mut self, bits: &[usize]) {
        bits.iter().for_each(|bit| self.remove_internal(*bit));
    }

    /// Trims all the [`BitBlocks`](`BitBlock`) at the end of the `BitSet` that contain no values.
    pub fn trim_trailing_zeros(&mut self) {
        while let Some(&0) = self.bits.last() {
            self.bits.pop();
        }
    }

    /// Checks wether or not this `BitSet` contains the nth `bit`
    pub fn contains<'a, 'b>(&'a self, bit: usize) -> bool {
        if let Some(block) = self.get_block(bit) {
            let bit = 1 << Self::bit_index(bit);
            return (*block & bit) == bit;
        } else {
            return false;
        }
    }

    /// Checks wether or not this `BitSet` contains all the bits on the `other` `BitSet`
    pub fn contains_all<'a, 'b>(&'a self, other: &'b BitSet) -> bool {
        let len = todo!("implement bitset & bitset");
    }
}

// region: Bitwise Operation Implementations
impl Not for BitSet {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        for block in self.bits.iter_mut() {
            *block = !*block;
        }
        self
    }
}

macro_rules! gen_binop_impls {
    ($self: ident, $common_bits: ident) => {};
    ([()] $self: ident, $common_bits: ident) => {
        $self.bits.truncate($common_bits);
    };
    ($type:ty[$op:tt $op2:tt] [$($truncate:tt)?] [$trait: ident]($trait_fn_name: ident) [$trait2: ident]($trait2_fn_name: ident)) => {
        impl $trait<&$type> for $type {
            type Output = $type;

            fn $trait_fn_name(mut self, rhs: &$type) -> Self::Output {
                let common_bits = usize::min(self.bits.len(), rhs.bits.len());
                self.bits[..common_bits]
                    .iter_mut()
                    .zip(rhs.bits[..common_bits].iter())
                    .for_each(|(a, b)| *a $op2 *b);

                gen_binop_impls!($([$truncate])? self, common_bits);
                self
            }
        }

        impl $trait<&$type> for &$type {
            type Output = $type;

            fn $trait_fn_name(self, rhs: &$type) -> Self::Output {
                self.clone() $op rhs // Reuse impl
            }
        }

        impl $trait2<&$type> for $type {
            fn $trait2_fn_name(&mut self, rhs: &$type) {
                let common_bits = usize::min(self.bits.len(), rhs.bits.len());
                self.bits[..common_bits]
                    .iter_mut()
                    .zip(rhs.bits[..common_bits].iter())
                    .for_each(|(a, b)| *a $op2 *b);

                gen_binop_impls!($([$truncate])? self, common_bits);
            }
        }
    };
}

gen_binop_impls!(BitSet[& &=] [()] [BitAnd](bitand) [BitAndAssign](bitand_assign));
gen_binop_impls!(BitSet[| |=] [  ] [BitOr](bitor)   [BitOrAssign](bitor_assign));
gen_binop_impls!(BitSet[^ ^=] [  ] [BitXor](bitxor) [BitXorAssign](bitxor_assign));

// endregion

// TODO: Impl PartialOrd + Ord for sorting bitsets into a binary heap for traversal and matching archetypes

#[cfg(test)]
mod tests {
    use super::{BitBlock, BitSet};

    #[test]
    fn new() {
        let set = BitSet::new();
        assert_eq!(true, set.is_empty());
    }

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
    fn add() {
        let mut set = BitSet::from_raw_bits(&[0b00000001]);
        assert_eq!(1, set.block_count());

        // Set the nth bit corresponding to the first bit of the second block, to grow the set.
        set.add(BitBlock::BITS as usize);
        assert_eq!(2, set.block_count());
    }

    #[test]
    fn add_all() {
        let mut set = BitSet::new();
        set.add_all(&[BitBlock::BITS as usize, 4, 0]);

        assert_eq!(2, set.block_count());
    }

    #[test]
    fn remove() {
        let mut set = BitSet::from_raw_bits(&[0b10000001]);
        assert_eq!(true, set.contains(0));

        set.remove(0);
        assert_eq!(false, set.contains(0));
    }

    #[test]
    fn remove_all() {
        let mut set = BitSet::from_raw_bits(&[0b10000001, 0b000000001]);
        assert_eq!(2, set.block_count());

        set.remove_all(&[0, BitBlock::BITS as usize]);
        assert_eq!(2, set.block_count());
        assert_eq!(false, set.contains(0));
        assert_eq!(false, set.contains(BitBlock::BITS as usize));
    }

    #[test]
    fn trim_trailing_zeros() {
        let mut set = BitSet::from_raw_bits(&[0b00000001, 0b00000001, 0, 0, 0b00000001]);
        assert_eq!(5, set.block_count());

        set.remove_all(&[(BitBlock::BITS * 4) as usize]);
        assert_eq!(5, set.block_count());

        set.trim_trailing_zeros();
        assert_eq!(2, set.block_count());
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
        todo!();
    }

    #[test]
    fn bitwise_not() {
        let mut set = BitSet::from_raw_bits(&[0b10000001]);
        set = !set;

        assert_eq!(BitBlock::MAX ^ 0b10000001, set.bits[0]);
    }

    const LHS: BitBlock = 0b10000011;
    const RHS: BitBlock = 0b10010001;

    fn get_lhs_rhs() -> (BitSet, BitSet) {
        (BitSet::from_raw_bits(&[LHS]), BitSet::from_raw_bits(&[RHS]))
    }

    #[test]
    fn bitwise_and() {
        // By Value and Ref: only lhs get consumed
        let (lhs, rhs) = get_lhs_rhs();
        let result = lhs & (&rhs);
        assert_eq!(LHS & RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Value and Mut : only lhs get consumed
        let (lhs, mut rhs) = get_lhs_rhs();
        let result = lhs & (&mut rhs);
        assert_eq!(LHS & RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Ref : new value generated/allocated
        let (lhs, rhs) = get_lhs_rhs();
        let result = (&lhs) & (&rhs);
        assert_eq!(LHS & RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }

    #[test]
    fn bitwise_and_assign() {
        // No Values consumed
        let (mut lhs, rhs) = get_lhs_rhs();

        lhs &= &rhs;
        assert_eq!(LHS & RHS, lhs.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }

    #[test]
    fn bitwise_or() {
        // By Value and Ref: only lhs get consumed
        let (lhs, rhs) = get_lhs_rhs();
        let result = lhs | (&rhs);
        assert_eq!(LHS | RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Value and Mut : only lhs get consumed
        let (lhs, mut rhs) = get_lhs_rhs();
        let result = lhs | (&mut rhs);
        assert_eq!(LHS | RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Ref : new value generated/allocated
        let (lhs, rhs) = get_lhs_rhs();
        let result = (&lhs) | (&rhs);
        assert_eq!(LHS | RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }

    #[test]
    fn bitwise_or_assign() {
        // No Values consumed
        let (mut lhs, rhs) = get_lhs_rhs();

        lhs |= &rhs;
        assert_eq!(LHS | RHS, lhs.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }

    #[test]
    fn bitwise_xor() {
        // By Value and Ref: only lhs get consumed
        let (lhs, rhs) = get_lhs_rhs();
        let result = lhs ^ (&rhs);
        assert_eq!(LHS ^ RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Value and Mut : only lhs get consumed
        let (lhs, mut rhs) = get_lhs_rhs();
        let result = lhs ^ (&mut rhs);
        assert_eq!(LHS ^ RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);

        // By Ref : new value generated/allocated
        let (lhs, rhs) = get_lhs_rhs();
        let result = (&lhs) ^ (&rhs);
        assert_eq!(LHS ^ RHS, result.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }

    #[test]
    fn bitwise_xor_assign() {
        // No Values consumed
        let (mut lhs, rhs) = get_lhs_rhs();

        lhs ^= &rhs;
        assert_eq!(LHS ^ RHS, lhs.bits[0]);
        assert_eq!(RHS, rhs.bits[0]);
    }
}
