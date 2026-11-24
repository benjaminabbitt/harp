//! Indexed and seeded-permutation access to the name space.
//!
//! A store that must never issue the same name twice keeps a `seed` and the
//! NEXT `position`, which starts at 0. A mint claims the name at `next` of a
//! seeded permutation of the name space and increments `next` in the same
//! unit of work, so names are unique by construction and any issued name can
//! be regenerated from `(SCHEME_VERSION, options, seed, position)`. The
//! sequence is exhausted when `next == size` ([`IndexError::OutOfRange`]);
//! it never wraps. The store deals only in integers; this module maps
//! integers to words.
//!
//! Options are validated strictly, here and in
//! [`generate_name_with_options`](crate::generate_name_with_options): 2 to 4
//! components (the largest space, about 8.98e12 names, fits `u64` and `i64`),
//! a length filter that leaves at least one adjective and one noun, and a
//! separator that is non-empty and holds no ASCII lowercase letter.
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
//!
//! # Hash-seeded names (what [`name_for`] promises)
//!
//! A caller with a stable input (a request id, say) wants the SAME name every
//! time that input is presented, with a collision settled by a claim rather
//! than by luck. [`name_for`] derives the seed from the input itself, and the
//! caller REQUESTS the hash size ([`HashSize`]: 64 or 128 bits):
//!
//! 1. `h = FNV-1a(input)` at the requested width: a `u64` for
//!    [`HashSize::Bits64`], a `u128` for [`HashSize::Bits128`].
//! 2. `seed = h as u64`: all of a 64-bit hash, the LOW 64 bits of a 128-bit
//!    one.
//! 3. The base name is `NameSpace::name_at_position(seed, 0)` for the
//!    caller's [`NameOptions`](crate::NameOptions) (scheme v1, above).
//! 4. `taken(base)` decides. A `true` answer appends `-` and the first 6
//!    lowercase hex digits of `h`, then 8, 10, .. digits, each tried in turn,
//!    up to the whole hash: 16 digits for 64 bits, 32 for 128. `h` is written
//!    as `width/4` lowercase hex digits, MOST significant first (the zero
//!    padded `{:016x}` or `{:032x}`). For 128 bits the suffix is the TOP of
//!    the hash while the seed is the BOTTOM, so two inputs whose seeds
//!    collide (same base name) still differ in the suffix unless all 128 bits
//!    agree; were the seed the high half, a seed collision would share its
//!    first 16 suffix digits and the ladder would be no use. With 64 bits the
//!    seed IS the hash, so a seed collision is a whole-hash collision and the
//!    ladder cannot separate two such inputs: ask for 128 bits where that
//!    matters. The suffix separator is always `-`, whatever
//!    `NameOptions::separator` joins the base with.
//! 5. `taken` answered `true` even for the whole hash:
//!    [`IndexError::HashExhausted`]. Never a silent random name.
//!
//! `taken` is `FnMut`, called in order, once per candidate, and stops at the
//! first `false`. In a caller it is the claim attempt itself (effectful), so
//! there is no check-then-claim race.
//!
//! **The hash is FNV-1a**, from rust-support's `fnv-hash` crate (`fnv1a_64`,
//! `fnv1a_128`), which documents its parameters, published vectors and how
//! the 128-bit vectors were derived. In short: Fowler-Noll-Vo variant 1a, XOR
//! each byte in, then multiply by the prime, wrapping; 64-bit offset basis
//! `0xcbf29ce484222325` and prime `0x100000001b3`, 128-bit offset basis
//! `0x6c62272e07bb014262b821756295c58d` and prime
//! `0x0000000001000000000000000000013B` (IETF draft-eastlake-fnv;
//! <http://www.isthe.com/chongo/tech/comp/fnv/>). It needs no dependency of
//! its own and is the same in every binding. It is NOT cryptographic: fine
//! for names, not for adversarial inputs, and a collision is caught by
//! `taken` anyway. **It is a compatibility promise**: changing the hash, its
//! parameters, which bits seed, or the suffix form renames every name ever
//! derived from an input, so it is part of scheme v1 and moves only with a
//! new [`SCHEME_VERSION`].

use crate::{NameOptions, ADJECTIVES, NOUNS};
use fnv_hash::{fnv1a_128, fnv1a_64};
use rand::Rng;

/// Version of the (word lists, index mapping, permutation) scheme. A store
/// records it beside each seed; a name regenerates only under the version
/// that issued it.
pub const SCHEME_VERSION: u32 = 1;

/// The fewest components a name has.
pub const MIN_COMPONENTS: u8 = 2;

/// The most components a name has: "anything more than that isn't really a
/// harp". The cap keeps every space inside `u64`.
pub const MAX_COMPONENTS: u8 = 4;

/// Why a name could not be made, or an indexed operation was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IndexError {
    /// `components` outside 2..=4. Refused rather than clamped, so a recorded
    /// option always means exactly one space.
    #[error("components must be 2..=4, got {0}")]
    InvalidComponents(u8),

    /// The `max_element_length` filter leaves no adjective or no noun.
    #[error("max_element_length {0} leaves an empty word list")]
    EmptyWordList(usize),

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

    /// [`name_for`] was answered "taken" for the base name and for every
    /// suffix up to the whole hash. `name` is the base name. The
    /// caller must not paper over it with a random name.
    #[error("every candidate for {name:?}, up to its whole-hash suffix, is taken")]
    HashExhausted { name: String },
}

/// The names for one `NameOptions`, numbered `0..size()`.
#[derive(Debug, Clone)]
pub struct NameSpace {
    adjectives: Vec<&'static str>,
    nouns: Vec<&'static str>,
    components: u8,
    separator: String,
    size: u64,
}

fn filtered(list: &[&'static str], max: Option<usize>) -> Vec<&'static str> {
    list.iter()
        .copied()
        .filter(|w| max.is_none_or(|m| w.len() <= m))
        .collect()
}

impl NameSpace {
    /// The space `opts` describes. Refuses (never clamps) components outside
    /// 2..=4, an emptying filter, and an ambiguous separator.
    pub fn new(opts: &NameOptions) -> Result<Self, IndexError> {
        if !(MIN_COMPONENTS..=MAX_COMPONENTS).contains(&opts.components) {
            return Err(IndexError::InvalidComponents(opts.components));
        }
        let adjectives = filtered(&ADJECTIVES, opts.max_element_length);
        let nouns = filtered(&NOUNS, opts.max_element_length);
        if let Some(max) = opts.max_element_length {
            if adjectives.is_empty() || nouns.is_empty() {
                return Err(IndexError::EmptyWordList(max));
            }
        }
        let sep = &opts.separator;
        if sep.is_empty() || sep.bytes().any(|b| b.is_ascii_lowercase()) {
            return Err(IndexError::InvalidSeparator(sep.clone()));
        }
        // Within the cap this cannot overflow: 1,269^3 * 4,396 < 2^44.
        let size =
            (adjectives.len() as u64).pow(u32::from(opts.components) - 1) * nouns.len() as u64;
        Ok(Self {
            adjectives,
            nouns,
            components: opts.components,
            separator: sep.clone(),
            size,
        })
    }

    /// Number of distinct names: `|A|^(components-1) * |N|`.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The name with mixed-radix index `index`. Bijective with `index_of`.
    pub fn name_at(&self, index: u64) -> Result<String, IndexError> {
        if index >= self.size {
            return Err(IndexError::OutOfRange {
                value: index,
                size: self.size,
            });
        }
        let (a, n) = (self.adjectives.len() as u64, self.nouns.len() as u64);
        let mut words = vec![""; usize::from(self.components)];
        let last = words.len() - 1;
        words[last] = self.nouns[(index % n) as usize];
        let mut rest = index / n;
        for slot in (0..last).rev() {
            words[slot] = self.adjectives[(rest % a) as usize];
            rest /= a;
        }
        Ok(words.join(&self.separator))
    }

    /// The index of `name`, splitting on this space's separator. Inverse of
    /// `name_at`; refuses any name not in this space.
    pub fn index_of(&self, name: &str) -> Result<u64, IndexError> {
        let words: Vec<&str> = name.split(self.separator.as_str()).collect();
        if words.len() != usize::from(self.components) {
            return Err(IndexError::WrongComponentCount {
                expected: self.components,
                found: words.len(),
            });
        }
        let last = words.len() - 1;
        let position = |slot: usize, list: &[&str]| {
            list.binary_search(&words[slot])
                .map(|i| i as u64)
                .map_err(|_| IndexError::UnknownWord {
                    slot,
                    word: words[slot].to_string(),
                })
        };
        let a = self.adjectives.len() as u64;
        let mut index = 0;
        for slot in 0..last {
            index = index * a + position(slot, &self.adjectives)?;
        }
        Ok(index * self.nouns.len() as u64 + position(last, &self.nouns)?)
    }

    /// The seeded permutation of this space's indices.
    pub fn permutation(&self, seed: u64) -> Permutation {
        Permutation::new(self.size, seed).expect("a valid space is never empty")
    }

    /// `name_at(permutation(seed).index_at(position))`: the name a store with
    /// this seed issues at `position`.
    pub fn name_at_position(&self, seed: u64, position: u64) -> Result<String, IndexError> {
        self.name_at(self.permutation(seed).index_at(position)?)
    }

    /// `permutation(seed).position_of(index_of(name))`: the position `name`
    /// occupies in this seed's sequence (e.g. for an imported name).
    pub fn position_of(&self, seed: u64, name: &str) -> Result<u64, IndexError> {
        self.permutation(seed).position_of(self.index_of(name)?)
    }

    /// A uniformly random name from this space.
    pub(crate) fn random_name(&self, rng: &mut impl Rng) -> String {
        self.name_at(rng.random_range(0..self.size))
            .expect("a drawn index is below the size")
    }
}

const ROUNDS: usize = 8;
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// The SplitMix64 finaliser.
fn mix64(mut z: u64) -> u64 {
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A keyed bijection on `[0, len)`: scheme v1's 8-round Feistel network with
/// cycle-walking. Constant memory; each call costs a few Feistel evaluations.
#[derive(Debug, Clone)]
pub struct Permutation {
    len: u64,
    half_bits: u32,
    keys: [u64; ROUNDS],
}

impl Permutation {
    /// The permutation of `[0, len)` for `seed`. `len` 0 is `EmptyDomain`.
    pub fn new(len: u64, seed: u64) -> Result<Self, IndexError> {
        if len == 0 {
            return Err(IndexError::EmptyDomain);
        }
        // The smallest even width w >= 2 with 2^w >= len.
        let bits = (u64::BITS - (len - 1).leading_zeros()).max(2);
        let width = bits + bits % 2;
        let mut keys = [0; ROUNDS];
        for (r, key) in keys.iter_mut().enumerate() {
            *key = mix64(seed.wrapping_add((r as u64 + 1).wrapping_mul(GOLDEN)));
        }
        Ok(Self {
            len,
            half_bits: width / 2,
            keys,
        })
    }

    /// The domain size.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Always false: an empty domain is refused at construction.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn mask(&self) -> u64 {
        (1 << self.half_bits) - 1
    }

    fn forward(&self, x: u64) -> u64 {
        let mask = self.mask();
        let (mut l, mut r) = (x >> self.half_bits, x & mask);
        for k in self.keys {
            (l, r) = (r, l ^ (mix64(r ^ k) & mask));
        }
        (l << self.half_bits) | r
    }

    fn backward(&self, x: u64) -> u64 {
        let mask = self.mask();
        let (mut l, mut r) = (x >> self.half_bits, x & mask);
        for k in self.keys.iter().rev() {
            (l, r) = (r ^ (mix64(l ^ k) & mask), l);
        }
        (l << self.half_bits) | r
    }

    fn walk(&self, value: u64, step: impl Fn(u64) -> u64) -> Result<u64, IndexError> {
        if value >= self.len {
            return Err(IndexError::OutOfRange {
                value,
                size: self.len,
            });
        }
        let mut x = step(value);
        while x >= self.len {
            x = step(x);
        }
        Ok(x)
    }

    /// The index at `position`; `position >= len` is `OutOfRange`.
    pub fn index_at(&self, position: u64) -> Result<u64, IndexError> {
        self.walk(position, |x| self.forward(x))
    }

    /// The position holding `index`; inverse of `index_at`, same cost.
    pub fn position_of(&self, index: u64) -> Result<u64, IndexError> {
        self.walk(index, |x| self.backward(x))
    }
}

/// A deterministic name for `seed` with default options: position 0 of the
/// seed's permutation, so it equals the first name a store with this seed
/// issues.
pub fn generate_name_with_seed(seed: u64) -> String {
    generate_name_with_options_and_seed(&NameOptions::default(), seed)
        .expect("the default options are valid")
}

/// A deterministic name for `seed` with `opts`: position 0 of the seed's
/// permutation of that space. Refuses what `NameSpace::new` refuses; never
/// clamps.
pub fn generate_name_with_options_and_seed(
    opts: &NameOptions,
    seed: u64,
) -> Result<String, IndexError> {
    NameSpace::new(opts)?.name_at_position(seed, 0)
}

/// The width of the hash that seeds [`name_for`], requested by the caller.
/// It sets where the seed comes from and how long the suffix ladder is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashSize {
    /// FNV-1a 64: the seed is the hash; the ladder ends at 16 hex digits.
    Bits64,
    /// FNV-1a 128: the seed is the low 64 bits; the ladder ends at 32 hex
    /// digits.
    Bits128,
}

/// The name for `input` under `opts`: position 0 of the permutation seeded
/// from the FNV-1a hash of `input` at `size`, then `-` plus 6, 8, .. hex
/// digits of the hash while `taken` answers `true`. `taken` is called in
/// order, once per candidate, and its first `false` ends the search; the whole
/// hash taken is [`IndexError::HashExhausted`]. Invalid options are refused
/// before `taken` is called. See the module docs.
pub fn name_for(
    input: &[u8],
    opts: &NameOptions,
    size: HashSize,
    taken: impl FnMut(&str) -> bool,
) -> Result<String, IndexError> {
    let mut taken = taken;
    let space = NameSpace::new(opts)?;
    let (hex, seed) = match size {
        HashSize::Bits64 => {
            let h = fnv1a_64(input);
            (format!("{h:016x}"), h)
        }
        HashSize::Bits128 => {
            let h = fnv1a_128(input);
            (format!("{h:032x}"), h as u64)
        }
    };
    let base = space.name_at_position(seed, 0)?;
    if !taken(&base) {
        return Ok(base);
    }
    for digits in (6..=hex.len()).step_by(2) {
        let candidate = format!("{base}-{}", &hex[..digits]);
        if !taken(&candidate) {
            return Ok(candidate);
        }
    }
    Err(IndexError::HashExhausted { name: base })
}
