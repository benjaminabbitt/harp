# Harp

**H**uman **A**ppropriate **R**andom **P**hraselets

Generate memorable, human-readable random names from adjectives and nouns.

```
swift-amber-falcon      quiet-silver-meadow     bold-crimson-thunder
misty-golden-river      warm-velvet-horizon     keen-azure-summit
```

## Why Harp?

- **Words people know**: 1,269 adjectives × 4,396 nouns = 5.6 million combinations using only familiar words
- **Readable output**: No random strings, UUIDs, or hex—just words humans can remember and type
- **One implementation**: Rust core shared across all platforms via FFI and WASM
- **Zero config**: Works out of the box with sensible defaults

Use cases: container names, temporary credentials, session IDs, test fixtures, branch names, anything that needs a memorable identifier.

## Word Lists

The word lists are curated for **familiarity**—words a college-educated English speaker would recognize immediately.

**Approach:**

1. Start with [EFF Diceware](https://www.eff.org/dice) (7,776 words) — curated for word recognition, concreteness, and memorability
2. Tag parts of speech using [Moby Project](https://en.wikipedia.org/wiki/Moby_Project) dictionary (233K words with POS)
3. Intersect to get EFF words properly classified as nouns or adjectives
4. Filter adjectives to exclude words that are also nouns (avoid "cardboard-shoe")

**Result:**

| Category | Count | Examples |
|----------|-------|----------|
| Adjectives | 1,269 | blissful, cautious, geometric, playful, rigid |
| Nouns | 4,396 | falcon, meadow, thunder, horizon, summit |

This gives 5.6 million unique name combinations while ensuring every word is immediately recognizable.

## Quick Start

### Rust

```toml
[dependencies]
harp-core = "0.1"
```

```rust
use harp_core::generate_name;

let name = generate_name();  // "swift-amber-falcon"
```

### Python

```bash
pip install harp-names
```

```python
import harp

name = harp.generate_name()  # "swift-amber-falcon"
```

### Go

```bash
go get github.com/benjaminabbitt/harp/bindings/go
```

```go
import "github.com/benjaminabbitt/harp/bindings/go"

name := harp.GenerateName()  // "swift-amber-falcon"
```

### JavaScript / TypeScript

```bash
npm install @babbitt/harp-wasm
```

```javascript
import init, { generate_name } from '@babbitt/harp-wasm';

await init();
const name = generate_name();  // "swift-amber-falcon"
```

### CLI (Rust)

```bash
cargo install harp-cli
```

```bash
$ harp
swift-amber-falcon
```

### CLI (Go)

```bash
go install github.com/benjaminabbitt/harp/bindings/go/cmd/harp@latest
```

```bash
$ harp
swift-amber-falcon

$ harp -c 2           # 2 components
bold-falcon

$ harp -s "_"         # custom separator
swift_amber_falcon
```

## Options

All bindings support the same configuration, and every binding validates it
strictly: options outside these rules are **refused, never clamped** (an
error in Rust and Go, `ValueError` in Python, a thrown `Error` in JavaScript,
NULL from the C ABI).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `components` | 2-4 | 3 | Number of words (N-1 adjectives + 1 noun) |
| `max_element_length` | uint | none | Maximum characters per word; a limit that leaves no adjective or no noun is refused |
| `separator` | string | `"-"` | Delimiter between words; must be non-empty and contain no `a-z`, so a name splits back into its words one way |

### Rust

```rust
use harp_core::{generate_name_with_options, NameOptions};

let opts = NameOptions {
    components: 2,
    max_element_length: Some(5),
    separator: "_".to_string(),
};
let name = generate_name_with_options(&opts)?;  // "bold_hawk"
```

### Python

```python
name = harp.generate_name_with_options(
    components=2,
    max_element_length=5,
    separator="_"
)  # "bold_hawk"
```

### Go

```go
opts := harp.Options{
    Components:       2,
    MaxElementLength: 5,
    Separator:        "_",
}
name, err := harp.GenerateNameWithOptions(opts)  // "bold_hawk"
```

## Seeded, unique names (Rust)

`harp_core::indexed` numbers every name in a space and walks it in a seeded
order, so a store can issue names that are unique by construction, with no
retry on collision:

```rust
use harp_core::{NameOptions, NameSpace};

let space = NameSpace::new(&NameOptions { components: 2, ..Default::default() })?;
// A store keeps (SCHEME_VERSION, options, seed) and the NEXT position,
// starting at 0. Each mint claims `next` and increments it; `next == size()`
// means the sequence is exhausted (it never wraps).
let first = space.name_at_position(42, 0)?;   // "rigid-chapter"
assert_eq!(space.position_of(42, &first)?, 0);
```

`generate_name_with_seed(seed)` and `generate_name_with_options_and_seed(opts,
seed)` return position 0 of the seed's sequence. `name_at`/`index_of` map
between names and their index in alphabetical order; a name whose words are
not in the lists is refused (`UnknownWord`). The scheme (word lists, index
mapping, and an 8-round Feistel permutation) is versioned by `SCHEME_VERSION`.

Python exposes the seeded forms as `harp.generate_name_with_seed(*, seed)` and
`harp.generate_name_with_options_and_seed(*, seed, components=3,
max_element_length=None, separator="-")`, keyword-only, with
`harp.SCHEME_VERSION`; a seed that is not an `int` in `0..2**64` is refused. The other bindings do not
expose seeded naming yet.

### Hash-seeded names with a claim callback (Rust)

`harp_core::name_for(input, &opts, taken)` derives a name from a stable input
(its bytes, not a digest) so the same input gets the same name every time:

```rust,ignore
use harp_core::{name_for, NameOptions};

// `taken` is the caller's claim attempt: it answers true when the candidate
// is refused. harp calls it in order, once per candidate, until it says false.
let name = name_for(b"request-id", &NameOptions::default(), |candidate| {
    !try_to_claim(candidate)
})?;   // "brisk-copper-moth", else "brisk-copper-moth-3fa9c1", then 8, 10, .. 32 hex digits
```

How it works: `h = fnv1a_128(input)`; the seed is the LOW 64 bits of `h`; the
base name is position 0 of that seed's permutation (scheme v1) with your
`NameOptions`. When `taken` refuses a candidate, `-` plus the first 6, then 8,
10, .., 32 lowercase hex digits of `h` (most significant first) are appended;
all 32 refused is `IndexError::HashExhausted`. The suffix separator is always
`-`, whatever `NameOptions::separator` is.

**FNV-1a** (Fowler-Noll-Vo, variant 1a) starts from an offset basis and, for
each byte, XORs the byte in and then multiplies by the FNV prime, wrapping.
The 128-bit parameters are offset basis `0x6c62272e07bb014262b821756295c58d`
and prime `0x0000000001000000000000000000013B` (2^88 + 0x13B), from the IETF
draft-eastlake-fnv and <http://www.isthe.com/chongo/tech/comp/fnv/>. Vectors:
`""` is `6c62272e07bb014262b821756295c58d`, `"a"` is
`d228cb696f1a8caf78912b704e4a8964`, `"foobar"` is
`343e1662793c64bf6f0d3597ba446f18`. It needs no dependency and is identical in
every binding. It is not cryptographic (fine for names, not for adversarial
inputs; a collision is caught by `taken`), and it is a compatibility promise:
changing it renames every name derived from an input. taskloom's own 64-bit
FNV-1a for task ids is separate and unchanged. Only Rust exposes `name_for`
so far.

## Architecture

Single Rust implementation, multiple bindings:

```
┌─────────────────────────────────────────────────────────┐
│                       harp-core                         │
│            (Rust library, word lists, RNG)              │
└───────────┬───────────┬───────────┬───────────┬─────────┘
            │           │           │           │
     ┌──────┴──┐   ┌────┴────┐  ┌───┴───┐  ┌────┴────┐
     │ harp-cli│   │harp-ffi │  │harp-  │  │harp-wasm│
     │  (bin)  │   │  (C ABI)│  │python │  │(wasm32) │
     └─────────┘   └────┬────┘  │(PyO3) │  └────┬────┘
                        │       └───────┘       │
                   C/C++/Go*              Browser/Node.js
                                               │
                                          ┌────┴────┐
                                          │   Go    │
                                          │(wazero) │
                                          └─────────┘

* Go can use either FFI (cgo) or embedded WASM (pure Go, no CGO)
```

The Go binding embeds a ~375KB WASM binary and uses the wazero runtime—pure Go with no native dependencies.

## Building

Requires [just](https://github.com/casey/just).

```bash
just build          # Build Rust crates
just test           # Run tests
just build-python   # Build Python wheel (requires maturin)
just build-wasm     # Build WASM (requires wasm-pack)
just build-wasm-core && just test-go  # Build and test Go bindings
just --list         # Show all commands
```

### Docker

Build without local toolchain:

```bash
just docker-all     # Build everything
just docker-rust    # Rust + CLI
just docker-python  # Python wheel
just docker-go      # Go bindings
just docker-wasm    # WASM packages
```

## Project Structure

```
crates/
├── harp-core         # Core library (words, generation)
├── harp-cli          # Command-line tool
├── harp-ffi          # C FFI for native bindings
├── harp-python       # PyO3 Python bindings
├── harp-wasm         # wasm-bindgen for browser/Node
└── harp-wasm-core    # WASI WASM for Go runtime

bindings/
├── go/               # Go package (embeds WASM)
├── typescript/       # npm package
└── python/           # PyPI metadata
```

## License

BSD-3-Clause

---

*Name credit: [AdamPIcode](https://old.reddit.com/user/AdamPIcode)*
