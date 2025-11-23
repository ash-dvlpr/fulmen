use core::sync::atomic::{AtomicIsize, Ordering};
use core::{fmt, hash::Hash};

/// Unique ID of an `Entity`, that also acts as a handle.
///
/// Obtained by calling [`World::spawn`] to create new entities.
///
/// The IDs are a kind of [`generational indices`]; an `index`/`id` + a `generation`.
/// This allows trivially checking if an Entity reference is still valid,
/// by checking that the current `generation` for that entity `id` matches the one provided.
///
/// [`generational indices`]: https://lucassardois.medium.com/generational-indices-guide-8e3c5f7fd594
#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Entity {
    // Do not reorder the fields. The ordering is explicitly used by repr(C)
    // to make this struct equivalent to a u64 in the eyes of LLVM.
    id: u32,
    generation: u32,
}

impl Entity {
    /// A dummy `Entity` that acts as an "uninitialized" Entity.
    pub(crate) const INVALID: Entity = Entity {
        id: u32::MAX,
        generation: 0,
    };

    /// The `id` of the `Entity`.
    #[inline(always)]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// The `generation` of the `Entity`.
    #[inline(always)]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Creates a new `Entity` from a raw [`u32`] entity `id` and a `generation`.
    #[inline(always)]
    pub(crate) const fn from_raw_id_and_generation(id: u32, generation: u32) -> Entity {
        Entity { id, generation }
    }

    /// Creates a new `Entity` from a raw [`u32`] entity id.
    #[inline(always)]
    pub(crate) const fn from_raw_id(id: u32) -> Entity {
        Entity {
            id,
            ..Entity::INVALID
        }
    }

    /// Convert the entity to it's [`u64`] representation.
    #[inline(always)]
    pub const fn to_bits(&self) -> u64 {
        ((self.id as u64) << u32::BITS) | (self.generation as u64)
    }

    /// Reconstruct an `Entity` that was converted to a [`u64`] using [`Self::to_bits`].
    pub const fn from_bits(bits: u64) -> Option<Entity> {
        Some(Entity {
            id: (bits >> u32::BITS) as u32,
            generation: bits as u32,
        })
    }
}

impl PartialEq for Entity {
    #[inline]
    fn eq(&self, other: &Entity) -> bool {
        // Use `Self::to_bits` to trivialize the comparison, as `Entity` is basically a u64.
        self.to_bits() == other.to_bits()
    }
}

impl Eq for Entity {}

impl PartialOrd for Entity {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Use `Self::cmp` as it should be optimized.
        Some(self.cmp(other))
    }
}

impl Ord for Entity {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Use `Self::to_bits` to trivialize the comparison, as `Entity` is basically a u64.
        self.to_bits().cmp(&other.to_bits())
    }
}

impl Hash for Entity {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity [{}.v{}]", self.id, self.generation)
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity [{}.v{}]", self.id, self.generation)
    }
}

/// Holds some metadata about the currently managed [`entities`](`Entity`) inside of the [`World`].
///
/// This information includes the current `generation`, for tracking the validty of the [`Entity`] handles.
#[derive(Copy, Clone)]
pub(crate) struct EntityMeta {
    generation: u32,
    location: EntityLocation,
}

impl EntityMeta {
    pub(crate) const EMPTY: EntityMeta = EntityMeta {
        generation: 0,
        location: EntityLocation::INVALID,
    };
}

/// Describes the location of an `Entity` inside of an [`Archetype`].
#[derive(Copy, Clone)]
pub(crate) struct EntityLocation {
    pub archetype: u32, // TODO: Swap
    pub index: u32,
}

impl EntityLocation {
    pub(crate) const INVALID: EntityLocation = EntityLocation {
        archetype: u32::MAX, // TODO: Swap with `ArchetypeId::INVALID`
        index: u32::MAX,
    };
}

pub(crate) struct ReserveEntitiesIterator<'a> {
    // Reference to the entity metadata to recover the current generation.
    meta: &'a [EntityMeta],

    // Iterator over the IDs on the `free ids` to be reused.
    free_iter: core::slice::Iter<'a, u32>,

    // Iterator over the new IDs to be allocated.
    new_iter: core::ops::Range<u32>,
}

impl Iterator for ReserveEntitiesIterator<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.free_iter
            // If there are more IDs on the `free ids` range, reuse those IDs.
            .next()
            .map(|&id| Entity::from_raw_id_and_generation(id, self.meta[id as usize].generation))
            // Else, continue with the `new_ids` range.
            .or_else(|| self.new_iter.next().map(|id| Entity::from_raw_id(id)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.free_iter.len() + self.new_iter.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for ReserveEntitiesIterator<'_> {}

/// Storage for all the [`Entity`] metadata of the [`World`].
#[derive(Default)]
pub(crate) struct Entities {
    /// Metadata about all the entities that have existed during
    ///  the lifetime of the [`World`], active or not.
    meta: Vec<EntityMeta>,

    /// An `Vec` which holds all fo the `Entity` IDs, that have been freed, to be reused
    /// by new newly allocated ones, instead of creating new IDs.
    ///
    /// ```text
    /// pending *----------------------*
    ///         | free ids | requested |
    ///         *----------------------*
    ///        ^           ^           ^
    ///   new_ids  requested_cursor   pending.len()
    /// ```
    ///
    /// Using the `requested_cursor`, it represents 3 different sets of IDs:
    /// - *free ids*: The IDs from index `0..requested_cursor`.
    ///   These represent the IDs that are still available for new `Entity` allocations.
    ///
    /// - *requested ids*: The IDs from index `requested_cursor..pending.len()` to the end of the
    ///   `pending` Vec. These represent the IDs that have already been reserved for allocation.
    ///
    /// - *new ids*: In the case that `requested_cursor < 0`, the negative index
    ///   equals the amount of new IDs that should be generated.
    pending: Vec<u32>,
    requested_cursor: AtomicIsize,
}

impl Entities {
    /// Wether or not there pending work requiring `flush()` to be called.
    fn needs_flush(&mut self) -> bool {
        *self.requested_cursor.get_mut() != self.pending.len() as isize
    }

    /// Check that we do not have pending work requiring `flush()` to be called.
    #[inline]
    fn assert_flushed(&mut self) {
        debug_assert!(
            !self.needs_flush(),
            "flush() needs to be called before this operation is legal"
        );
    }

    /// Reserve `count` [`Entity`] IDs.
    ///
    /// The metadata for the new entities will be lazily allocated when `flush()` gets flushed.
    pub fn reserve_entities(&mut self, count: u32) -> ReserveEntitiesIterator {
        // The old `requested_cursor`, everything beyond that was already reserved,
        // so we decrement it again to reserve more entities via an atomic subtraction.
        let reserve_end = self.requested_cursor.fetch_sub(count as isize, Ordering::Relaxed);
        // We calculate the new value of `requested_cursor`, as we got the old pre-op value.
        let reserve_start = reserve_end - count as isize;

        // The range of the ids to be reused, after the atomic subtraction these will all
        // be inside of the reserved range, that's why we're using the old value from `fetch_sub`.
        let free_range = reserve_start.max(0) as usize..reserve_end.max(0) as usize;

        // Check wether or not we need to create new IDs, this will only need to be done if
        // `reserve_start` is negative, which means we need to create that many new IDs.
        let new_ids_range = if reserve_start >= 0 {
            0..0
        // Create the new IDs for the remaining entities.
        } else {
            let first_new_id = self.meta.len() as isize;

            // `-` * `-` == `+`, thus we are getting `first_new_id` + how many new allocations are needed
            let new_ids_end = u32::try_from(first_new_id - reserve_start)
                .expect("max amount of entities exceeded");

            // If `new_ids_end` is in range, the start will be smaller so it's in range.
            let new_ids_start = (first_new_id - reserve_start.min(0)) as u32;

            new_ids_start..new_ids_end
        };

        // Return an iterator to process the allocation of the entities, as this only reserves the IDs.
        ReserveEntitiesIterator {
            meta: &self.meta[..],
            free_iter: self.pending[free_range].iter(),
            new_iter: new_ids_range,
        }
    }

    /// Reserve one [`Entity`] ID.
    ///
    /// The metadata for the new entities will be lazily allocated when `flush()` gets flushed.
    ///
    /// Equivalent to `Self::reserve_entities(1).next()?`, but without redundant operations.
    pub fn reserve_entity(&mut self) -> Entity {
        let reserve_end = self.requested_cursor.fetch_sub(1, Ordering::Relaxed);
        // Try to reuse an ID from the `free ids` vec.
        if reserve_end > 0 {
            let index = self.pending[(reserve_end - 1) as usize];
            Entity::from_raw_id_and_generation(index, self.meta[index as usize].generation)
        // Generate a new ID.
        } else {
            Entity::from_raw_id(
                u32::try_from(self.meta.len() as isize - reserve_end)
                    .expect("max amount of entities exceeded"),
            )
        }
    }

    /// Allocate one `Entity` ID directly.
    pub fn alloc(&mut self) -> Entity {
        self.assert_flushed();
        todo!("directly allocate an entity");
    }

    /// Destroy an `Entity` allowing for it to be reused later.
    pub fn free(&mut self, entity: Entity) -> Entity {
        self.assert_flushed();
        todo!("free an entity");
    }

    // TODO: Contains entity

    // TODO: Get entity location
    // TODO: Set entity location (must call to update the cached location of an entity after moving it in storage)

    // TODO: flush
    // TODO: flush with dummy archetype

    // TODO: active_count       (meta.len() - pending.len())
    // TODO: used_count         (meta.len() + reserved)
    // TODO: total_count        (meta.len())
    // TODO: total_prediction   (meta.len() + amount allocated if `flush`)
}
