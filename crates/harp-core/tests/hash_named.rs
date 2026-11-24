//! Tests for hash-seeded names with a claim callback ([`harp_core::name_for`]).
//!
//! The FNV-1a 128-bit parameters are published (IETF draft-eastlake-fnv); the
//! hash vectors here were computed by an independent model of the loop. They
//! are a compatibility promise: every name derived from an input depends on
//! them, so a change must fail here loudly.

use harp_core::{fnv1a_128, name_for, IndexError, NameOptions, NameSpace};

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

fn hex(input: &[u8]) -> String {
    format!("{:032x}", fnv1a_128(input))
}

/// The base name the scheme promises: position 0 of the seed's permutation,
/// the seed being the low 64 bits of the hash.
fn base_of(input: &[u8], o: &NameOptions) -> String {
    let seed = fnv1a_128(input) as u64;
    NameSpace::new(o)
        .expect("valid options")
        .name_at_position(seed, 0)
        .expect("position 0 is in range")
}

/// Run `name_for` against a `taken` that answers true for the first `n`
/// candidates, returning the result and every candidate it was asked about.
fn ladder(
    input: &[u8],
    o: &NameOptions,
    taken_count: usize,
) -> (Result<String, IndexError>, Vec<String>) {
    let mut asked = Vec::new();
    let result = name_for(input, o, |c| {
        asked.push(c.to_string());
        asked.len() <= taken_count
    });
    (result, asked)
}

// ------------------------------------------------------- the hash itself

#[test]
fn test_fnv1a_128_of_empty_input_is_the_offset_basis() {
    assert_eq!(hex(b""), "6c62272e07bb014262b821756295c58d");
}

#[test]
fn test_fnv1a_128_matches_published_vector_for_a() {
    assert_eq!(hex(b"a"), "d228cb696f1a8caf78912b704e4a8964");
}

#[test]
fn test_fnv1a_128_matches_published_vector_for_foobar() {
    assert_eq!(hex(b"foobar"), "343e1662793c64bf6f0d3597ba446f18");
}

#[test]
fn test_fnv1a_128_is_xor_then_multiply_not_fnv1() {
    // FNV-1 (multiply, then xor) of "a" differs from FNV-1a's.
    let fnv1 = (0x6c62272e07bb014262b821756295c58du128
        .wrapping_mul(0x0000000001000000000000000000013B))
        ^ 0x61;
    assert_ne!(fnv1a_128(b"a"), fnv1);
}

// ---------------------------------------------------------- the seed bits

#[test]
fn test_seed_is_the_low_64_bits_of_the_hash() {
    // "foobar": hash 343e1662793c64bf6f0d3597ba446f18, so seed 6f0d3597ba446f18.
    let o = default_opts();
    let expected = NameSpace::new(&o)
        .unwrap()
        .name_at_position(0x6f0d_3597_ba44_6f18, 0)
        .unwrap();
    let (name, asked) = ladder(b"foobar", &o, 0);
    assert_eq!(name.unwrap(), expected);
    assert_eq!(asked, vec![expected]);
}

#[test]
fn test_seed_is_not_the_high_64_bits() {
    let o = default_opts();
    let high = NameSpace::new(&o)
        .unwrap()
        .name_at_position(0x343e_1662_793c_64bf, 0)
        .unwrap();
    let (name, _) = ladder(b"foobar", &o, 0);
    assert_ne!(name.unwrap(), high);
}

// ------------------------------------------------- determinism and spread

#[test]
fn test_same_input_gives_the_same_name() {
    let o = default_opts();
    let a = name_for(b"req-1", &o, |_| false).unwrap();
    let b = name_for(b"req-1", &o, |_| false).unwrap();
    assert_eq!(a, b);
    assert_eq!(a, base_of(b"req-1", &o));
}

#[test]
fn test_different_inputs_generally_give_different_names() {
    let o = default_opts();
    let names: std::collections::HashSet<String> = (0..200)
        .map(|i| name_for(format!("req-{i}").as_bytes(), &o, |_| false).unwrap())
        .collect();
    // 200 draws from ~1.1e10 names: a collision is not expected.
    assert_eq!(names.len(), 200);
}

// ------------------------------------------------------------ the ladder

#[test]
fn test_free_base_name_is_returned_after_one_call() {
    let o = default_opts();
    let (name, asked) = ladder(b"req-1", &o, 0);
    let base = base_of(b"req-1", &o);
    assert_eq!(name.unwrap(), base);
    assert_eq!(asked, vec![base]);
}

#[test]
fn test_base_taken_once_returns_it_with_a_six_hex_suffix() {
    let o = default_opts();
    let base = base_of(b"req-1", &o);
    let h = hex(b"req-1");
    let (name, asked) = ladder(b"req-1", &o, 1);
    let want = format!("{base}-{}", &h[..6]);
    assert_eq!(name.unwrap(), want);
    assert_eq!(asked, vec![base, want]);
}

#[test]
fn test_suffix_grows_six_then_eight_hex_digits() {
    let o = default_opts();
    let base = base_of(b"req-1", &o);
    let h = hex(b"req-1");
    let (name, asked) = ladder(b"req-1", &o, 2);
    let want = format!("{base}-{}", &h[..8]);
    assert_eq!(name.unwrap(), want);
    assert_eq!(
        asked,
        vec![base.clone(), format!("{base}-{}", &h[..6]), want]
    );
}

#[test]
fn test_suffix_steps_by_two_up_to_the_whole_hash_in_order() {
    let o = default_opts();
    let base = base_of(b"req-1", &o);
    let h = hex(b"req-1");
    let mut expected = vec![base.clone()];
    for digits in (6..=32).step_by(2) {
        expected.push(format!("{base}-{}", &h[..digits]));
    }
    assert_eq!(expected.len(), 15);
    // Everything but the last candidate is taken.
    let (name, asked) = ladder(b"req-1", &o, 14);
    assert_eq!(name.unwrap(), expected[14]);
    assert_eq!(asked, expected);
    assert!(expected[14].ends_with(&h));
}

#[test]
fn test_suffix_is_lowercase_hex_most_significant_digit_first() {
    let o = default_opts();
    let (name, _) = ladder(b"foobar", &o, 1);
    assert!(name.unwrap().ends_with("-343e16"));
}

#[test]
fn test_all_32_hex_digits_taken_is_a_typed_error_naming_the_base() {
    let o = default_opts();
    let base = base_of(b"req-1", &o);
    let (result, asked) = ladder(b"req-1", &o, usize::MAX);
    assert_eq!(result, Err(IndexError::HashExhausted { name: base }));
    assert_eq!(
        asked.len(),
        15,
        "the base and 14 suffixes, then it gives up"
    );
}

#[test]
fn test_taken_is_not_called_again_after_a_false_answer() {
    let o = default_opts();
    let (_, asked) = ladder(b"req-1", &o, 3);
    assert_eq!(asked.len(), 4);
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
    let first = name_for(b"req-1", &o, &mut claim).unwrap();
    let second = name_for(b"req-2", &o, &mut claim).unwrap();
    let again = name_for(b"req-1", &o, &mut claim).unwrap();
    assert_eq!(first, base_of(b"req-1", &o));
    assert_eq!(second, base_of(b"req-2", &o));
    // The retry finds its own name claimed and falls to the six-digit suffix.
    assert_eq!(again, format!("{first}-{}", &hex(b"req-1")[..6]));
}

// -------------------------------------------------------------- options

#[test]
fn test_components_option_sets_the_word_count_of_the_base() {
    for components in 2..=4u8 {
        let o = opts(components, None, "-");
        let name = name_for(b"req-1", &o, |_| false).unwrap();
        assert_eq!(name.split('-').count(), usize::from(components));
        assert_eq!(name, base_of(b"req-1", &o));
    }
}

#[test]
fn test_separator_option_joins_the_base_but_the_suffix_is_always_a_hyphen() {
    let o = opts(2, None, "_");
    let base = base_of(b"req-1", &o);
    assert!(base.contains('_'));
    let (name, asked) = ladder(b"req-1", &o, 1);
    assert_eq!(asked[0], base);
    assert_eq!(name.unwrap(), format!("{base}-{}", &hex(b"req-1")[..6]));
}

#[test]
fn test_max_element_length_option_is_honoured_in_the_base() {
    let o = opts(2, Some(3), "_");
    let name = name_for(b"req-1", &o, |_| false).unwrap();
    assert!(name.split('_').all(|w| w.len() <= 3));
    assert_eq!(name, base_of(b"req-1", &o));
}

#[test]
fn test_invalid_options_are_refused_before_taken_is_called() {
    let mut called = false;
    let result = name_for(b"req-1", &opts(5, None, "-"), |_| {
        called = true;
        false
    });
    assert_eq!(result, Err(IndexError::InvalidComponents(5)));
    assert!(!called);
}
