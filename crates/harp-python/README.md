# harp-names

Generate random, human-readable names from adjectives and nouns.

## Installation

```bash
pip install harp-names
```

## Usage

```python
import harp

# Generate a random name (default: 3 components)
name = harp.generate_name()
print(name)  # e.g., "misty-golden-river"

# Generate with options
name = harp.generate_name_with_options(
    components=2,              # 2-4 words
    max_element_length=5,      # max chars per word (None = no limit)
    separator="_"              # custom separator
)
print(name)  # e.g., "bold_hawk"

# Deterministic names from a seed (an int in 0..2**64, passed as seed=)
harp.generate_name_with_seed(seed=42)                             # "outdated-statutory-paramount"
harp.generate_name_with_options_and_seed(seed=42, components=2)   # "rigid-chapter"
```

A seeded name is position 0 of the seed's permutation of the name space
(scheme `harp.SCHEME_VERSION`); the same seed and options give the same name
here as in harp-core (Rust). Only harp-core and this Python binding have seeded
naming; the Go, FFI and TypeScript/WASM bindings do not expose it.

## API

- `generate_name()` - Generate a random name (2 adjectives + 1 noun)
- `generate_name_with_options(components=3, max_element_length=None, separator="-")` - Generate with options; raises `ValueError` for components outside 2-4, a length that leaves no word, or a separator that is empty or contains a-z
- `generate_name_with_seed(*, seed)` - The name for `seed` with the default options
- `generate_name_with_options_and_seed(*, seed, components=3, max_element_length=None, separator="-")` - The name for `seed` with options; refuses the options as `generate_name_with_options` does
- Both seeded forms take every argument by keyword only, so `seed` is always named: a positional argument raises `TypeError`
- Both seeded forms raise `TypeError` for a seed that is not an `int` (a `bool` is refused) and `ValueError` for one outside `0..2**64`; nothing is wrapped or clamped
- `SCHEME_VERSION` - The seeded scheme's version; record it beside a stored seed
- `version()` - Get the library version

## License

BSD-3-Clause
