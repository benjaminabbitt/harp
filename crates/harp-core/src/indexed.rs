//! Indexed and seeded-permutation access to the name space.
//!
//! RED PROPOSAL: every body here is a stub. The contracts below, and the
//! fixed vectors in `tests/seeded_index.rs`, are what is under review.
//!
//! A store that must never issue the same name twice keeps a `seed` and the
//! most recent `position`. Each mint advances the position by one and takes
//! the name at that position of a seeded permutation of the name space, so
//! names are unique by construction and any issued name can be regenerated
//! from `(SCHEME_VERSION, options, seed, position)`. The store deals only in
//! integers; this module maps integers to words.
//!
//! # Scheme v1 (what [`SCHEME_VERSION`] = 1 promises)
//!
//! **Lists.** [`ADJECTIVES`] and [`NOUNS`] in their shipped order (strictly
//! ascending, unique). `max_element_length = Some(m)` keeps the words of at
//! most `m` bytes, in list order; the filtered lists ARE the space, so a
//! store must record `max_element_length` beside the seed.
//!
//! **Index.** For `c` components over adjectives `A` and nouns `N`, the space
//! has `|A|^(c-1) * |N|` names. A name `a_1 .. a_(c-1) n` has the mixed-radix
//! index `((a_1 * |A| + a_2) * |A| + .. + a_(c-1)) * |N| + n`, where each word
//! stands for its position in its filtered list. Index order is therefore the
//! lexicographic order of names: index 0 is the first adjective repeated with
//! the first noun. The separator only joins and splits words; it is not part
//! of the index.
//!
//! **Permutation** over `[0, n)` for a `u64` seed:
//! - `mix64(z)`: the SplitMix64 finaliser, `z ^= z >> 30; z *= 0xBF58476D1CE4E5B9;
//!   z ^= z >> 27; z *= 0x94D049BB133111EB; z ^= z >> 31` (wrapping, u64).
//! - Round keys `k_r = mix64(seed + (r + 1) * 0x9E3779B97F4A7C15)` (wrapping)
//!   for `r` in `0..8`.
//! - Width `w`: the smallest EVEN number of bits, at least 2, with `2^w >= n`;
//!   half width `h = w / 2`, `mask = 2^h - 1`.
//! - Balanced Feistel on `[0, 2^w)`: split `x` into `L = x >> h`,
//!   `R = x & mask`; for `r` in `0..8`, `(L, R) = (R, L ^ (mix64(R ^ k_r) & mask))`;
//!   result `(L << h) | R`. The inverse runs the rounds backwards:
//!   `(L, R) = (R ^ (mix64(L ^ k_r) & mask), L)`.
//! - Cycle-walking: `index_at(p)` applies the Feistel to `p`, and again to its
//!   own output, until the value is below `n`. `position_of` does the same
//!   with the inverse. Because `2^w < 4n`, a call expects fewer than four
//!   Feistel evaluations, and nothing proportional to `n` is materialised.
//!
//! Any change to the lists, the index mapping or the permutation is a new
//! scheme and must bump [`SCHEME_VERSION`]; the fixed vectors fail otherwise.

use crate::{NameOptions, ADJECTIVES, NOUNS};

/// Version of the (word lists, index mapping, permutation) scheme. A store
/// records it beside each seed; a name regenerates only under the version
/// that issued it.
pub const SCHEME_VERSION: u32 = 1;

/// Why an indexed operation was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IndexError {
    /// `components` outside 2..=16. Indexed access refuses rather than
    /// clamps, so a recorded option always means exactly one space.
    #[error("components must be 2..=16, got {0}")]
    InvalidComponents(u8),

    /// The `max_element_length` filter leaves no adjective or no noun.
    #[error("max_element_length {0} leaves an empty word list")]
    EmptyWordList(usize),

    /// The space has more than `u64::MAX` names (unfiltered: more than 6
    /// components).
    #[error("the name space for {components} components does not fit in u64")]
    SpaceTooLarge { components: u8 },

    /// The separator is empty or contains an ASCII lowercase letter, either of
    /// which could make a name split into words in more than one way.
    #[error("separator {0:?} cannot be split unambiguously")]
    InvalidSeparator(String),

    /// An index or position at or beyond the size of the space. For a
    /// position this means the seeded sequence is exhausted.
    #[error("{value} is out of range for a space of {size}")]
    OutOfRange { value: u64, size: u64 },

    /// The permutation domain is empty.
    #[error("a permutation needs a domain of at least one element")]
    EmptyDomain,

    /// `index_of` was given a name with the wrong number of words.
    #[error("expected {expected} words, found {found}")]
    WrongComponentCount { expected: u8, found: usize },

    /// A word is not in the (filtered) list for its slot: an adjective slot
    /// holding a noun, a word from another list (e.g. Go ctxloom's "default"
    /// group), or one longer than `max_element_length`. A store treats such a
    /// name as taken but unindexed.
    #[error("word {word:?} at slot {slot} is not in this space")]
    UnknownWord { slot: usize, word: String },
}

/// The names for one `NameOptions`, numbered `0..size()`.
#[derive(Debug, Clone)]
pub struct NameSpace {
    adjectives: Vec<&'static str>,
    nouns: Vec<&'static str>,
    components: u8,
    separator: String,
}

impl NameSpace {
    /// The space `opts` describes. Refuses (never clamps) out-of-range
    /// components, an emptying filter, a space past `u64`, and an ambiguous
    /// separator.
    pub fn new(opts: &NameOptions) -> Result<Self, IndexError> {
        let _ = (opts, &*ADJECTIVES, &*NOUNS);
        todo!("NameSpace::new")
    }

    /// Number of distinct names: `|A|^(components-1) * |N|`.
    pub fn size(&self) -> u64 {
        let _ = (
            &self.adjectives,
            &self.nouns,
            self.components,
            &self.separator,
        );
        todo!("NameSpace::size")
    }

    /// The name with mixed-radix index `index`. Bijective with `index_of`.
    pub fn name_at(&self, index: u64) -> Result<String, IndexError> {
        let _ = index;
        todo!("NameSpace::name_at")
    }

    /// The index of `name`, splitting on this space's separator. Inverse of
    /// `name_at`; refuses any name not in this space.
    pub fn index_of(&self, name: &str) -> Result<u64, IndexError> {
        let _ = name;
        todo!("NameSpace::index_of")
    }

    /// The seeded permutation of this space's indices.
    pub fn permutation(&self, seed: u64) -> Permutation {
        let _ = seed;
        todo!("NameSpace::permutation")
    }

    /// `name_at(permutation(seed).index_at(position))`: the name a store with
    /// this seed issues at `position`.
    pub fn name_at_position(&self, seed: u64, position: u64) -> Result<String, IndexError> {
        let _ = (seed, position);
        todo!("NameSpace::name_at_position")
    }

    /// `permutation(seed).position_of(index_of(name))`: the position `name`
    /// occupies in this seed's sequence (e.g. for an imported name).
    pub fn position_of(&self, seed: u64, name: &str) -> Result<u64, IndexError> {
        let _ = (seed, name);
        todo!("NameSpace::position_of")
    }
}

/// A keyed bijection on `[0, len)`: scheme v1's 8-round Feistel network with
/// cycle-walking. Constant memory; each call costs a few Feistel evaluations.
#[derive(Debug, Clone)]
pub struct Permutation {
    len: u64,
    half_bits: u32,
    keys: [u64; 8],
}

impl Permutation {
    /// The permutation of `[0, len)` for `seed`. `len` 0 is `EmptyDomain`.
    pub fn new(len: u64, seed: u64) -> Result<Self, IndexError> {
        let _ = (len, seed);
        todo!("Permutation::new")
    }

    /// The domain size.
    pub fn len(&self) -> u64 {
        let _ = (self.len, self.half_bits, self.keys);
        todo!("Permutation::len")
    }

    /// Always false: an empty domain is refused at construction.
    pub fn is_empty(&self) -> bool {
        todo!("Permutation::is_empty")
    }

    /// The index at `position`; `position >= len` is `OutOfRange`.
    pub fn index_at(&self, position: u64) -> Result<u64, IndexError> {
        let _ = position;
        todo!("Permutation::index_at")
    }

    /// The position holding `index`; inverse of `index_at`, same cost.
    pub fn position_of(&self, index: u64) -> Result<u64, IndexError> {
        let _ = index;
        todo!("Permutation::position_of")
    }
}

/// A deterministic name for `seed` with default options: position 0 of the
/// seed's permutation (see the question on seed semantics in the proposal).
pub fn generate_name_with_seed(seed: u64) -> String {
    let _ = seed;
    todo!("generate_name_with_seed")
}

/// A deterministic name for `seed` with `opts`: components are clamped to
/// 2..=16 as `generate_name_with_options` does, then position 0 of the seed's
/// permutation of that space. Refuses what `NameSpace::new` refuses.
pub fn generate_name_with_options_and_seed(
    opts: &NameOptions,
    seed: u64,
) -> Result<String, IndexError> {
    let _ = (opts, seed);
    todo!("generate_name_with_options_and_seed")
}
