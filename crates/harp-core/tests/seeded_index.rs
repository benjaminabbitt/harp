//! Tests for indexed and seeded-permutation naming (scheme v1), and the
//! strict option validation every generator shares.
//!
//! The fixed vectors were computed by an independent Python model of the
//! scheme written in `harp_core::indexed`'s module docs. They pin the word
//! lists, the index mapping and the permutation together: a name issued
//! today must regenerate byte-for-byte from (version, options, seed,
//! position), so any change to any of the three must fail here loudly and
//! bump `SCHEME_VERSION`.

use harp_core::{
    generate_name_with_options, generate_name_with_options_and_seed, generate_name_with_seed,
    IndexError, NameOptions, NameSpace, Permutation, ADJECTIVES, NOUNS, SCHEME_VERSION,
};
use std::collections::HashSet;

fn opts(components: u8, max_element_length: Option<usize>, separator: &str) -> NameOptions {
    NameOptions {
        components,
        max_element_length,
        separator: separator.to_string(),
    }
}

fn space(components: u8) -> NameSpace {
    NameSpace::new(&opts(components, None, "-")).expect("valid options")
}

/// The small space used for exhaustive checks: words of at most 3 letters
/// leave 2 adjectives ("icy", "sly") and 70 nouns.
fn small(components: u8) -> NameSpace {
    NameSpace::new(&opts(components, Some(3), "_")).expect("valid options")
}

// ---------------------------------------------------------------- guards
// These pass today. They hold the inputs the vectors depend on still.

/// FNV-1a 64 over the bytes, local to the test so it cannot drift with the
/// code under test.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn scheme_v1_pins_the_word_lists() {
    assert_eq!(SCHEME_VERSION, 1);
    assert_eq!((ADJECTIVES.len(), NOUNS.len()), (1269, 4396));
    assert_eq!(
        fnv1a(ADJECTIVES.join("\n").as_bytes()),
        0x362f_fe8c_b710_a394
    );
    assert_eq!(fnv1a(NOUNS.join("\n").as_bytes()), 0xcf21_0a30_3432_fb8b);
}

#[test]
fn word_lists_are_strictly_ascending_lowercase_and_disjoint() {
    for list in [&*ADJECTIVES, &*NOUNS] {
        assert!(
            list.windows(2).all(|w| w[0] < w[1]),
            "not strictly ascending"
        );
        assert!(list
            .iter()
            .all(|w| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase())));
    }
    let adjectives: HashSet<_> = ADJECTIVES.iter().collect();
    assert!(NOUNS.iter().all(|n| !adjectives.contains(n)));
}

// ------------------------------------------------------------ the space

#[test]
fn size_is_the_product_of_the_list_lengths() {
    assert_eq!(space(2).size(), 5_578_524);
    assert_eq!(space(3).size(), 7_079_146_956);
    // The cap: four components is the largest space, and it fits u64 (and
    // i64, a SQLite INTEGER) with room to spare.
    assert_eq!(space(4).size(), 8_983_437_487_164);
    assert!(space(4).size() < i64::MAX as u64);
    assert_eq!(small(2).size(), 140);
    assert_eq!(small(3).size(), 280);
}

#[test]
fn more_than_four_components_are_refused() {
    // "Anything more than that isn't really a harp": the cap is 4, so no
    // accepted space can outgrow u64.
    for c in [5, 6, 7, 16] {
        let err = NameSpace::new(&opts(c, None, "-")).unwrap_err();
        assert_eq!(err, IndexError::InvalidComponents(c));
    }
    assert!(NameSpace::new(&opts(4, None, "-")).is_ok());
}

#[test]
fn options_are_refused_not_clamped() {
    for c in [0, 1, 5, 17, 20] {
        let err = NameSpace::new(&opts(c, None, "-")).unwrap_err();
        assert_eq!(err, IndexError::InvalidComponents(c));
    }
    // No adjective is 2 letters or fewer.
    let err = NameSpace::new(&opts(2, Some(2), "-")).unwrap_err();
    assert_eq!(err, IndexError::EmptyWordList(2));
    for sep in ["", "x", "-a-"] {
        let err = NameSpace::new(&opts(2, None, sep)).unwrap_err();
        assert_eq!(err, IndexError::InvalidSeparator(sep.to_string()));
    }
}

#[test]
fn name_at_fixed_vectors() {
    let two = space(2);
    assert_eq!(two.name_at(0).unwrap(), "abdominal-abacus");
    assert_eq!(two.name_at(1).unwrap(), "abdominal-abdomen");
    assert_eq!(two.name_at(4396).unwrap(), "abiding-abacus");
    assert_eq!(two.name_at(2_789_262).unwrap(), "overblown-licking");
    assert_eq!(two.name_at(5_578_523).unwrap(), "zoning-zoom");

    let three = space(3);
    assert_eq!(three.name_at(0).unwrap(), "abdominal-abdominal-abacus");
    assert_eq!(three.name_at(1).unwrap(), "abdominal-abdominal-abdomen");
    assert_eq!(three.name_at(4396).unwrap(), "abdominal-abiding-abacus");
    assert_eq!(
        three.name_at(3_539_573_478).unwrap(),
        "overblown-overblown-licking"
    );
    assert_eq!(three.name_at(7_079_146_955).unwrap(), "zoning-zoning-zoom");
}

#[test]
fn index_of_fixed_vectors() {
    assert_eq!(space(2).index_of("zoning-zoom").unwrap(), 5_578_523);
    assert_eq!(
        space(3).index_of("abiding-abdominal-cake").unwrap(),
        5_578_991
    );
    let underscored = NameSpace::new(&opts(2, None, "_")).unwrap();
    assert_eq!(underscored.index_of("rigid_chapter").unwrap(), 3_323_974);
}

#[test]
fn name_at_refuses_an_index_past_the_end() {
    let two = space(2);
    assert_eq!(
        two.name_at(5_578_524).unwrap_err(),
        IndexError::OutOfRange {
            value: 5_578_524,
            size: 5_578_524
        }
    );
}

#[test]
fn index_of_refuses_names_outside_the_space() {
    let two = space(2);
    let unknown = |name: &str, slot: usize, word: &str| {
        assert_eq!(
            two.index_of(name).unwrap_err(),
            IndexError::UnknownWord {
                slot,
                word: word.to_string()
            },
            "{name}"
        );
    };
    unknown("swift-falcon", 0, "swift"); // the README's example is not in the lists
    unknown("abacus-abacus", 0, "abacus"); // a noun in an adjective slot
    unknown("abdominal-abdominal", 1, "abdominal"); // an adjective in the noun slot
    assert_eq!(
        two.index_of("abdominal-abiding-abacus").unwrap_err(),
        IndexError::WrongComponentCount {
            expected: 2,
            found: 3
        }
    );
    // Go ctxloom's "default" group: "calm" is a harp-core NOUN, so a Go
    // project id like this is taken but unindexed.
    assert_eq!(
        space(3).index_of("calm-bald-cake").unwrap_err(),
        IndexError::UnknownWord {
            slot: 0,
            word: "calm".to_string()
        }
    );
    // A word the filter removed is outside the filtered space.
    assert_eq!(
        small(2).index_of("abdominal_abacus").unwrap_err(),
        IndexError::UnknownWord {
            slot: 0,
            word: "abdominal".to_string()
        }
    );
}

#[test]
fn name_at_and_index_of_are_a_bijection_over_a_small_space() {
    for components in [2, 3] {
        let sp = small(components);
        let mut seen = HashSet::new();
        for i in 0..sp.size() {
            let name = sp.name_at(i).unwrap();
            assert_eq!(sp.index_of(&name).unwrap(), i, "{name}");
            let words: Vec<&str> = name.split('_').collect();
            assert_eq!(words.len(), usize::from(components));
            assert!(words.iter().all(|w| w.len() <= 3), "{name}");
            assert!(seen.insert(name), "duplicate at {i}");
        }
        assert_eq!(seen.len() as u64, sp.size());
    }
}

#[test]
fn name_at_and_index_of_round_trip_across_the_full_space() {
    for components in [2, 3, 4] {
        let sp = space(components);
        let n = sp.size();
        for i in [0, 1, 2, n / 3, n / 2, n - 2, n - 1] {
            assert_eq!(sp.index_of(&sp.name_at(i).unwrap()).unwrap(), i);
        }
    }
}

// ---------------------------------------------------------- permutation

#[test]
fn permutation_covers_its_domain_exactly_once() {
    let mut lens: Vec<u64> = (1..=300).collect();
    lens.extend([1000, 4096, 4097]);
    for len in lens {
        for seed in [0, 1, 42, u64::MAX] {
            let p = Permutation::new(len, seed).unwrap();
            assert_eq!(p.len(), len);
            assert!(!p.is_empty());
            let mut hit = vec![false; len as usize];
            for pos in 0..len {
                let idx = p.index_at(pos).unwrap();
                assert!(idx < len, "len {len} seed {seed}: {idx} escaped");
                assert!(!hit[idx as usize], "len {len} seed {seed}: {idx} twice");
                hit[idx as usize] = true;
                assert_eq!(p.position_of(idx).unwrap(), pos, "inverse");
            }
        }
    }
}

#[test]
fn permutation_refuses_an_empty_domain_and_out_of_range_input() {
    assert_eq!(Permutation::new(0, 7).unwrap_err(), IndexError::EmptyDomain);
    let p = Permutation::new(10, 7).unwrap();
    let past = IndexError::OutOfRange {
        value: 10,
        size: 10,
    };
    assert_eq!(p.index_at(10).unwrap_err(), past);
    assert_eq!(p.position_of(10).unwrap_err(), past);
}

#[test]
fn a_seed_reorders_but_is_deterministic() {
    let a = Permutation::new(1000, 1).unwrap();
    let b = Permutation::new(1000, 1).unwrap();
    let c = Permutation::new(1000, 2).unwrap();
    let order = |p: &Permutation| {
        (0..1000)
            .map(|i| p.index_at(i).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&a), order(&b));
    assert_ne!(order(&a), order(&c));
    assert_ne!(order(&a), (0..1000).collect::<Vec<_>>(), "not the identity");
}

/// (seed, position, name) for the 2-component space: taskloom's task harps.
const TASK_VECTORS: &[(u64, u64, &str)] = &[
    (0, 0, "nervy-overdraft"),
    (0, 1, "stuck-flinch"),
    (0, 2, "sporting-flight"),
    (0, 1000, "uncut-native"),
    (0, 5_578_523, "chatty-legend"),
    (1, 0, "certified-expert"),
    (1, 1, "available-engraver"),
    (1, 2, "livable-cartel"),
    (1, 1000, "panoramic-huff"),
    (1, 5_578_523, "unusable-armful"),
    (42, 0, "rigid-chapter"),
    (42, 1, "skinny-cardinal"),
    (42, 2, "capable-oblivion"),
    (42, 1000, "stark-dart"),
    (42, 5_578_523, "freehand-spew"),
    (0xDEAD_BEEF, 0, "turbulent-doorman"),
    (0xDEAD_BEEF, 1, "grievous-fog"),
    (0xDEAD_BEEF, 2, "relatable-reheat"),
    (0xDEAD_BEEF, 1000, "pregame-lankiness"),
    (0xDEAD_BEEF, 5_578_523, "posh-gigolo"),
    (u64::MAX, 0, "unburned-savanna"),
    (u64::MAX, 1, "sloppy-moneybags"),
    (u64::MAX, 2, "choosy-routine"),
    (u64::MAX, 1000, "petite-stimulus"),
    (u64::MAX, 5_578_523, "augmented-anointer"),
];

/// (seed, position, name) for the 3-component space: taskloom's project ids.
const PROJECT_VECTORS: &[(u64, u64, &str)] = &[
    (0, 0, "clumsy-turbulent-hypnotism"),
    (0, 1, "glowing-devious-mortician"),
    (0, 2, "moonlit-isolating-eardrum"),
    (0, 1000, "untold-aftermost-evidence"),
    (0, 7_079_146_955, "pliable-same-coagulant"),
    (1, 0, "bouncy-pregame-congress"),
    (1, 1, "untoasted-obsessive-bobtail"),
    (1, 2, "uncut-spiral-gondola"),
    (1, 1000, "unblessed-stipulate-magenta"),
    (1, 7_079_146_955, "manned-studied-stress"),
    (42, 0, "outdated-statutory-paramount"),
    (42, 1, "wasting-tall-banjo"),
    (42, 2, "factual-lucid-suburb"),
    (42, 1000, "hasty-unburned-document"),
    (42, 7_079_146_955, "happy-splendid-compare"),
    (0xDEAD_BEEF, 0, "reborn-uncivil-chess"),
    (0xDEAD_BEEF, 1, "childish-pungent-detention"),
    (0xDEAD_BEEF, 2, "enjoyable-patchy-decline"),
    (0xDEAD_BEEF, 1000, "growing-glamorous-distance"),
    (0xDEAD_BEEF, 7_079_146_955, "stung-jolly-quarrel"),
    (u64::MAX, 0, "angular-bogus-preview"),
    (u64::MAX, 1, "repeated-lustrous-disgrace"),
    (u64::MAX, 2, "energetic-unretired-quadrant"),
    (u64::MAX, 1000, "impotent-unscented-disposal"),
    (u64::MAX, 7_079_146_955, "latticed-sloppy-ember"),
];

fn check_vectors(sp: &NameSpace, vectors: &[(u64, u64, &str)]) {
    for &(seed, position, name) in vectors {
        assert_eq!(
            sp.name_at_position(seed, position).unwrap(),
            name,
            "seed {seed:#x} position {position}"
        );
        assert_eq!(
            sp.position_of(seed, name).unwrap(),
            position,
            "inverse of {name} under seed {seed:#x}"
        );
        let index = sp.index_of(name).unwrap();
        assert_eq!(sp.permutation(seed).index_at(position).unwrap(), index);
    }
}

#[test]
fn seeded_task_harps_fixed_vectors() {
    check_vectors(&space(2), TASK_VECTORS);
}

#[test]
fn seeded_project_ids_fixed_vectors() {
    check_vectors(&space(3), PROJECT_VECTORS);
}

#[test]
fn seeded_vectors_in_a_filtered_space_with_another_separator() {
    let sp = small(2);
    let first: Vec<String> = (0..3).map(|p| sp.name_at_position(7, p).unwrap()).collect();
    assert_eq!(first, ["icy_sip", "icy_tug", "icy_hut"]);
}

/// Positions are 0-based and a store keeps the NEXT one, starting at 0: a
/// mint claims `next` and increments it. The last name is at `size - 1`;
/// `next == size` means exhausted, and the sequence never wraps.
#[test]
fn positions_run_from_zero_to_size_minus_one() {
    let sp = small(2);
    let mut next = 0;
    let mut issued = Vec::new();
    while let Ok(name) = sp.name_at_position(7, next) {
        issued.push(name);
        next += 1;
    }
    assert_eq!(next, sp.size(), "exhausted exactly at next == size");
    assert_eq!(issued[0], sp.name_at_position(7, 0).unwrap());
    assert_eq!(sp.position_of(7, &issued[0]).unwrap(), 0);
    assert_eq!(
        sp.position_of(7, issued.last().unwrap()).unwrap(),
        sp.size() - 1
    );
}

#[test]
fn name_at_position_refuses_an_exhausted_sequence() {
    let sp = small(2);
    assert_eq!(
        sp.name_at_position(7, 140).unwrap_err(),
        IndexError::OutOfRange {
            value: 140,
            size: 140
        }
    );
}

#[test]
fn a_seeded_sequence_never_repeats_over_a_small_space() {
    let sp = small(3);
    let names: HashSet<String> = (0..sp.size())
        .map(|p| sp.name_at_position(99, p).unwrap())
        .collect();
    assert_eq!(names.len() as u64, sp.size());
}

// ------------------------------------------------- seeded generate_name*

#[test]
fn generate_name_with_seed_is_position_zero_of_the_default_space() {
    assert_eq!(generate_name_with_seed(42), "outdated-statutory-paramount");
    assert_eq!(generate_name_with_seed(0), "clumsy-turbulent-hypnotism");
    assert_eq!(generate_name_with_seed(42), generate_name_with_seed(42));
}

#[test]
fn generate_name_with_options_and_seed_is_position_zero_and_strict() {
    assert_eq!(
        generate_name_with_options_and_seed(&opts(2, None, "-"), 42).unwrap(),
        "rigid-chapter"
    );
    assert_eq!(
        generate_name_with_options_and_seed(&opts(2, None, "_"), 42).unwrap(),
        "rigid_chapter"
    );
    assert_eq!(
        generate_name_with_options_and_seed(&opts(2, None, "-"), 42).unwrap(),
        space(2).name_at_position(42, 0).unwrap()
    );
    // Strict, as generate_name_with_options is: refused, never clamped.
    for c in [1, 5, 20] {
        assert_eq!(
            generate_name_with_options_and_seed(&opts(c, None, "-"), 42).unwrap_err(),
            IndexError::InvalidComponents(c)
        );
    }
    assert_eq!(
        generate_name_with_options_and_seed(&opts(2, Some(2), "-"), 42).unwrap_err(),
        IndexError::EmptyWordList(2)
    );
    assert_eq!(
        generate_name_with_options_and_seed(&opts(2, None, ""), 42).unwrap_err(),
        IndexError::InvalidSeparator(String::new())
    );
}

// ------------------------------------------ strict generate_name_with_options

#[test]
fn generate_name_with_options_refuses_what_the_space_refuses() {
    for c in [0, 1, 5, 16, 20] {
        assert_eq!(
            generate_name_with_options(&opts(c, None, "-")).unwrap_err(),
            IndexError::InvalidComponents(c)
        );
    }
    assert_eq!(
        generate_name_with_options(&opts(3, Some(2), "-")).unwrap_err(),
        IndexError::EmptyWordList(2)
    );
    for sep in ["", "x", "-a-"] {
        assert_eq!(
            generate_name_with_options(&opts(2, None, sep)).unwrap_err(),
            IndexError::InvalidSeparator(sep.to_string())
        );
    }
}

#[test]
fn generate_name_with_options_draws_from_the_space_it_validated() {
    for c in [2, 3, 4] {
        let sp = NameSpace::new(&opts(c, Some(5), " ")).unwrap();
        let name = generate_name_with_options(&opts(c, Some(5), " ")).unwrap();
        // A space separator is valid (full-sail's ship names use it).
        assert_eq!(name.split(' ').count(), usize::from(c), "{name}");
        assert!(sp.index_of(&name).is_ok(), "{name} is outside its space");
    }
}
