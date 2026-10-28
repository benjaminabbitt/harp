"""The Python binding's seeded naming: harp_core's generate_name_with_seed and
generate_name_with_options_and_seed, under the binding's strict refusals.

The fixed vectors are harp-core's (crates/harp-core/tests/seeded_index.rs):
a seeded name is position 0 of the seed's scheme-v1 permutation, so it must
agree with Rust byte for byte.
"""

import pytest

import harp


def test_scheme_version_is_exposed():
    assert harp.SCHEME_VERSION == 1


def test_generate_name_with_seed_matches_the_core_vectors():
    assert harp.generate_name_with_seed(seed=42) == "outdated-statutory-paramount"
    assert harp.generate_name_with_seed(seed=0) == "clumsy-turbulent-hypnotism"


def test_generate_name_with_seed_takes_the_whole_u64_range():
    top = harp.generate_name_with_seed(seed=2**64 - 1)
    assert len(top.split("-")) == 3
    assert harp.generate_name_with_seed(seed=2**64 - 1) == top


@pytest.mark.parametrize(
    "call",
    [harp.generate_name_with_seed, harp.generate_name_with_options_and_seed],
)
def test_seed_is_keyword_only(call):
    # A positional int could be mistaken for another argument; seed= says what it is.
    with pytest.raises(TypeError):
        call(42)
    assert call(seed=42) == "outdated-statutory-paramount"


def test_options_after_the_seed_are_keyword_only_too():
    with pytest.raises(TypeError):
        harp.generate_name_with_options_and_seed(2, seed=42)


def test_generate_name_with_options_and_seed_matches_the_core_vectors():
    assert harp.generate_name_with_options_and_seed(seed=42, components=2) == "rigid-chapter"
    assert (
        harp.generate_name_with_options_and_seed(seed=42, components=2, separator="_")
        == "rigid_chapter"
    )


def test_generate_name_with_options_and_seed_defaults_are_the_default_space():
    assert harp.generate_name_with_options_and_seed(seed=42) == harp.generate_name_with_seed(seed=42)


def test_max_element_length_shapes_the_seeded_space():
    name = harp.generate_name_with_options_and_seed(seed=7, components=4, max_element_length=4, separator=" ")
    words = name.split(" ")
    assert len(words) == 4
    assert all(len(w) <= 4 for w in words), name


@pytest.mark.parametrize("components", [0, 1, 5, 20])
def test_components_outside_2_to_4_are_refused(components):
    with pytest.raises(ValueError, match=f"components must be 2..=4, got {components}"):
        harp.generate_name_with_options_and_seed(seed=42, components=components)


def test_an_emptying_length_filter_is_refused():
    with pytest.raises(ValueError, match="leaves an empty word list"):
        harp.generate_name_with_options_and_seed(seed=42, components=2, max_element_length=2)


@pytest.mark.parametrize("separator", ["", "x", "-a-"])
def test_an_ambiguous_separator_is_refused(separator):
    with pytest.raises(ValueError, match="cannot be split unambiguously"):
        harp.generate_name_with_options_and_seed(seed=42, separator=separator)


@pytest.mark.parametrize("seed", [-1, 2**64, -(2**70)])
@pytest.mark.parametrize(
    "call",
    [harp.generate_name_with_seed, harp.generate_name_with_options_and_seed],
)
def test_a_seed_outside_u64_is_refused_not_wrapped(call, seed):
    with pytest.raises(ValueError, match="seed must be an integer in 0..2\\*\\*64"):
        call(seed=seed)


@pytest.mark.parametrize("seed", [True, False, 1.0, "42", None])
@pytest.mark.parametrize(
    "call",
    [harp.generate_name_with_seed, harp.generate_name_with_options_and_seed],
)
def test_a_seed_that_is_not_an_int_is_refused(call, seed):
    with pytest.raises(TypeError, match="seed must be an int"):
        call(seed=seed)


def test_the_seed_is_required():
    with pytest.raises(TypeError):
        harp.generate_name_with_seed()
    with pytest.raises(TypeError):
        harp.generate_name_with_options_and_seed(components=2)
