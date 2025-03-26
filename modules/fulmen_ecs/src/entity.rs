use core::sync::atomic::{AtomicIsize, Ordering};
use core::{fmt, hash::Hash, num::NonZeroU32};

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
    generation: NonZeroU32,
}

impl Entity {
    /// A dummy `Entity` that acts as an "uninitialized" Entity.
    pub(crate) const DUMMY: Entity = Entity {
        id: u32::MAX,
        generation: NonZeroU32::MIN,
    };

    /// The `id` of the `Entity`.
    #[inline(always)]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// The `generation` of the `Entity`.
    #[inline(always)]
    pub fn generation(&self) -> u32 {
        self.generation.get()
    }

    /// Creates a new `Entity` from a raw [`u32`] entity `id` and a `generation`.
    #[inline(always)]
    pub(crate) const fn from_raw_id_and_generation(id: u32, generation: NonZeroU32) -> Entity {
        Entity { id, generation }
    }

    /// Creates a new `Entity` from a raw [`u32`] entity id.
    #[inline(always)]
    pub(crate) const fn from_raw_id(id: u32) -> Entity {
        Entity {
            id,
            ..Entity::DUMMY
        }
    }

    /// Convert the entity to it's [`u64`] representation.
    #[inline(always)]
    pub const fn to_bits(&self) -> u64 {
        ((self.id as u64) << u32::BITS) | (self.generation.get() as u64)
    }

    /// Reconstruct an `Entity` that was converted to a [`u64`] using [`Self::to_bits`].
    pub const fn from_bits(bits: u64) -> Option<Entity> {
        Some(Entity {
            id: (bits >> u32::BITS) as u32,
            generation: match NonZeroU32::new(bits as u32) {
                Some(g) => g,
                None => return None,
            },
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
        write!(f, "Entity [{}.v{}]", self.id, self.generation.get())
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity [{}.v{}]", self.id, self.generation.get())
    }
}
