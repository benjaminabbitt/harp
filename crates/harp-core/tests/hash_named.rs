//! Tests for hash-seeded names with a claim callback ([`harp_core::name_for`]).
//!
//! The hash is FNV-1a, which will come from rust-support's `fnv-hash` crate
//! once it is published and pinned (harp-core does not depend on it yet, so
//! it stays buildable). Until then each test carries its own model of the
//! loop (`Hash::of`), pinned to the published vectors below, so the tests
//! state the scheme without calling the code under test for the hash. The
//! parameters and vectors are a compatibility promise: every name derived
//! from an input depends on them.

use harp_core::{name_for, HashSize, IndexError, NameOptions, NameSpace};

const SIZES: [HashSize; 2] = [HashSize::Bits64, HashSize::Bits128];

/// A hash at a width: the value and its width in bits.
struct Hash {
    value: u128,
    bits: u32,
}

impl Hash {
    fn of(input: &[u8], size: HashSize) -> Self {
        match size {
            HashSize::Bits64 => Hash {
                value: u128::from(input.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                    (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
                })),
                bits: 64,
            },
            HashSize::Bits128 => Hash {
                value: input
                    .iter()
                    .fold(0x6c62_272e_07bb_0142_62b8_2175_6295_c58du128, |h, b| {
                        (h ^ u128::from(*b)).wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013B)
                    }),
                bits: 128,
            },
        }
    }

    /// All digits, lowercase, zero padded, most significant first.
    fn hex(&self) -> String {
        format!("{:0width$x}", self.value, width = (self.bits / 4) as usize)
    }

    /// The seed: all of a 64-bit hash, the low 64 bits of a 128-bit one.
    fn seed(&self) -> u64 {
        self.value as u64
    }

    /// The suffix digit counts after the base: 6, 8, .. the whole hash.
    fn ladder(&self) -> Vec<usize> {
        (6..=(self.bits / 4) as usize).step_by(2).collect()
    }
}

fn opts(components: u8, max_element_length: Option<usize>, separator: &str) -> NameOptions {
    NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    }
}

fn default_opts() -> NameOptions {
    opts(3, None, "-")
}

/// The base name the scheme promises: position 0 of the seed's permutation.
fn base_of(input: &[u8], size: HashSize, o: &NameOptions) -> String {
    NameSpace::new(o)
        .expect("valid options")
        .name_at_position(Hash::of(input, size).seed(), 0)
        .expect("position 0 is in range")
}

/// Every candidate in order: the base, then base-<6 digits>, <8>, .. <whole>.
fn candidates(input: &[u8], size: HashSize, o: &NameOptions) -> Vec<String> {
    let h = Hash::of(input, size);
    let base = base_of(input, size, o);
    let mut all = vec![base.clone()];
    all.extend(
        h.ladder()
            .iter()
            .map(|d| format!("{base}-{}", &h.hex()[..*d])),
    );
    all
}

/// Run `name_for` against a `taken` that answers true for the first
/// `taken_count` candidates, returning the result and every candidate asked.
fn run(
    input: &[u8],
    o: &NameOptions,
    size: HashSize,
    taken_count: usize,
) -> (Result<String, IndexError>, Vec<String>) {
    let mut asked = Vec::new();
    let result = name_for(input, o, size, |c| {
        asked.push(c.to_string());
        asked.len() <= taken_count
    });
    (result, asked)
}

// ----------------------------------------- the model, pinned (guards)
// These pass today: they hold the model the other tests lean on.

#[test]
fn test_model_matches_the_published_64_bit_vectors() {
    let h = |s: &[u8]| Hash::of(s, HashSize::Bits64).hex();
    assert_eq!(h(b""), "cbf29ce484222325");
    assert_eq!(h(b"a"), "af63dc4c8601ec8c");
    assert_eq!(h(b"foobar"), "85944171f73967e8");
}

#[test]
fn test_model_matches_the_modelled_128_bit_vectors() {
    let h = |s: &[u8]| Hash::of(s, HashSize::Bits128).hex();
    assert_eq!(h(b""), "6c62272e07bb014262b821756295c58d");
    assert_eq!(h(b"a"), "d228cb696f1a8caf78912b704e4a8964");
    assert_eq!(h(b"foobar"), "343e1662793c64bf6f0d3597ba446f18");
}

// ---------------------------------------------------------- the seed bits

#[test]
fn test_seed_of_a_64_bit_hash_is_the_whole_hash() {
    // "foobar": hash 85944171f73967e8.
    let o = default_opts();
    let want = NameSpace::new(&o)
        .unwrap()
        .name_at_position(0x8594_4171_f739_67e8, 0)
        .unwrap();
    let (name, asked) = run(b"foobar", &o, HashSize::Bits64, 0);
    assert_eq!(name.unwrap(), want);
    assert_eq!(asked, vec![want]);
}

#[test]
fn test_seed_of_a_128_bit_hash_is_its_low_64_bits() {
    // "foobar": hash 343e1662793c64bf6f0d3597ba446f18, seed 6f0d3597ba446f18.
    let o = default_opts();
    let want = NameSpace::new(&o)
        .unwrap()
        .name_at_position(0x6f0d_3597_ba44_6f18, 0)
        .unwrap();
    let (name, asked) = run(b"foobar", &o, HashSize::Bits128, 0);
    assert_eq!(name.unwrap(), want);
    assert_eq!(asked, vec![want]);
}

#[test]
fn test_128_bit_seed_is_not_the_high_64_bits() {
    let o = default_opts();
    let high = NameSpace::new(&o)
        .unwrap()
        .name_at_position(0x343e_1662_793c_64bf, 0)
        .unwrap();
    let (name, _) = run(b"foobar", &o, HashSize::Bits128, 0);
    assert_ne!(name.unwrap(), high);
}

#[test]
fn test_the_requested_size_changes_the_name() {
    let o = default_opts();
    let a = name_for(b"foobar", &o, HashSize::Bits64, |_| false).unwrap();
    let b = name_for(b"foobar", &o, HashSize::Bits128, |_| false).unwrap();
    assert_ne!(a, b);
}

// ------------------------------------------------- determinism and spread

#[test]
fn test_same_input_and_size_give_the_same_name() {
    let o = default_opts();
    for size in SIZES {
        let a = name_for(b"req-1", &o, size, |_| false).unwrap();
        let b = name_for(b"req-1", &o, size, |_| false).unwrap();
        assert_eq!(a, b);
        assert_eq!(a, base_of(b"req-1", size, &o));
    }
}

#[test]
fn test_different_inputs_generally_give_different_names() {
    let o = default_opts();
    for size in SIZES {
        let names: std::collections::HashSet<String> = (0..200)
            .map(|i| name_for(format!("req-{i}").as_bytes(), &o, size, |_| false).unwrap())
            .collect();
        // 200 draws from ~1.1e10 names: a collision is not expected.
        assert_eq!(names.len(), 200);
    }
}

// ------------------------------------------------------------ the ladder

#[test]
fn test_free_base_name_is_returned_after_one_call() {
    let o = default_opts();
    for size in SIZES {
        let (name, asked) = run(b"req-1", &o, size, 0);
        let base = base_of(b"req-1", size, &o);
        assert_eq!(name.unwrap(), base);
        assert_eq!(asked, vec![base]);
    }
}

#[test]
fn test_base_taken_once_returns_it_with_a_six_hex_suffix() {
    let o = default_opts();
    for size in SIZES {
        let c = candidates(b"req-1", size, &o);
        let (name, asked) = run(b"req-1", &o, size, 1);
        assert_eq!(name.unwrap(), c[1]);
        assert_eq!(asked, c[..2].to_vec());
        assert!(c[1].ends_with(&Hash::of(b"req-1", size).hex()[..6]));
    }
}

#[test]
fn test_suffix_grows_six_then_eight_hex_digits() {
    let o = default_opts();
    for size in SIZES {
        let c = candidates(b"req-1", size, &o);
        let (name, asked) = run(b"req-1", &o, size, 2);
        assert_eq!(name.unwrap(), c[2]);
        assert_eq!(asked, c[..3].to_vec());
    }
}

#[test]
fn test_the_ladder_ends_at_16_hex_for_64_bits_and_32_for_128() {
    let o = default_opts();
    let (n64, asked64) = run(b"req-1", &o, HashSize::Bits64, 6);
    let c64 = candidates(b"req-1", HashSize::Bits64, &o);
    assert_eq!(c64.len(), 7, "the base and 6, 8, 10, 12, 14, 16 digits");
    assert_eq!(asked64, c64);
    assert!(n64
        .unwrap()
        .ends_with(&Hash::of(b"req-1", HashSize::Bits64).hex()));

    let (n128, asked128) = run(b"req-1", &o, HashSize::Bits128, 14);
    let c128 = candidates(b"req-1", HashSize::Bits128, &o);
    assert_eq!(c128.len(), 15, "the base and 6, 8, .. 32 digits");
    assert_eq!(asked128, c128);
    assert!(n128
        .unwrap()
        .ends_with(&Hash::of(b"req-1", HashSize::Bits128).hex()));
}

#[test]
fn test_suffix_is_lowercase_hex_most_significant_digit_first() {
    let o = default_opts();
    let (n128, _) = run(b"foobar", &o, HashSize::Bits128, 1);
    assert!(n128.unwrap().ends_with("-343e16"));
    let (n64, _) = run(b"foobar", &o, HashSize::Bits64, 1);
    assert!(n64.unwrap().ends_with("-859441"));
}

#[test]
fn test_suffix_keeps_leading_zeros_of_the_hash() {
    // Some short input hashes with a leading zero digit at 64 bits: find one,
    // so the zero padding is exercised without hard coding a vector.
    let o = default_opts();
    let input = (0u32..100_000)
        .map(|i| i.to_string().into_bytes())
        .find(|i| Hash::of(i, HashSize::Bits64).hex().starts_with('0'))
        .expect("about 1 in 16 inputs hashes with a leading zero");
    let (name, _) = run(&input, &o, HashSize::Bits64, 1);
    assert!(name
        .unwrap()
        .ends_with(&Hash::of(&input, HashSize::Bits64).hex()[..6]));
}

#[test]
fn test_whole_hash_taken_is_a_typed_error_naming_the_base() {
    let o = default_opts();
    for size in SIZES {
        let (result, asked) = run(b"req-1", &o, size, usize::MAX);
        assert_eq!(
            result,
            Err(IndexError::HashExhausted {
                name: base_of(b"req-1", size, &o)
            })
        );
        assert_eq!(asked, candidates(b"req-1", size, &o));
    }
}

#[test]
fn test_taken_is_not_called_again_after_a_false_answer() {
    let o = default_opts();
    for size in SIZES {
        let (_, asked) = run(b"req-1", &o, size, 3);
        assert_eq!(asked.len(), 4);
    }
}

#[test]
fn test_taken_may_carry_effects_and_is_called_in_order() {
    let o = default_opts();
    let mut claimed: Vec<String> = Vec::new();
    // A claim: refuses a name already claimed, records the ones it accepts.
    let mut claim = |c: &str| {
        if claimed.iter().any(|x| x == c) {
            true
        } else {
            claimed.push(c.to_string());
            false
        }
    };
    let first = name_for(b"req-1", &o, HashSize::Bits128, &mut claim).unwrap();
    let second = name_for(b"req-2", &o, HashSize::Bits128, &mut claim).unwrap();
    let again = name_for(b"req-1", &o, HashSize::Bits128, &mut claim).unwrap();
    assert_eq!(first, base_of(b"req-1", HashSize::Bits128, &o));
    assert_eq!(second, base_of(b"req-2", HashSize::Bits128, &o));
    // The retry finds its own name claimed and falls to the six-digit suffix.
    assert_eq!(again, candidates(b"req-1", HashSize::Bits128, &o)[1]);
}

// -------------------------------------------------------------- options

#[test]
fn test_components_option_sets_the_word_count_of_the_base() {
    for size in SIZES {
        for components in 2..=4u8 {
            let o = opts(components, None, "-");
            let name = name_for(b"req-1", &o, size, |_| false).unwrap();
            assert_eq!(name.split('-').count(), usize::from(components));
            assert_eq!(name, base_of(b"req-1", size, &o));
        }
    }
}

#[test]
fn test_separator_option_joins_the_base_but_the_suffix_is_always_a_hyphen() {
    let o = opts(2, None, "_");
    for size in SIZES {
        let base = base_of(b"req-1", size, &o);
        assert!(base.contains('_'));
        let (name, asked) = run(b"req-1", &o, size, 1);
        assert_eq!(asked[0], base);
        assert_eq!(
            name.unwrap(),
            format!("{base}-{}", &Hash::of(b"req-1", size).hex()[..6])
        );
    }
}

#[test]
fn test_max_element_length_option_is_honoured_in_the_base() {
    let o = opts(2, Some(3), "_");
    for size in SIZES {
        let name = name_for(b"req-1", &o, size, |_| false).unwrap();
        assert!(name.split('_').all(|w| w.len() <= 3));
        assert_eq!(name, base_of(b"req-1", size, &o));
    }
}

#[test]
fn test_invalid_options_are_refused_before_taken_is_called() {
    for size in SIZES {
        let mut called = false;
        let result = name_for(b"req-1", &opts(5, None, "-"), size, |_| {
            called = true;
            false
        });
        assert_eq!(result, Err(IndexError::InvalidComponents(5)));
        assert!(!called);
    }
}
