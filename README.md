# Harp

**H**uman **A**ppropriate **R**andom **P**hraselets

Generate memorable, human-readable random names from adjectives and nouns.

```
swift-amber-falcon      quiet-silver-meadow     bold-crimson-thunder
misty-golden-river      warm-velvet-horizon     keen-azure-summit
```

## Why Harp?

- **Massive vocabulary**: 17,874 adjectives × 55,191 nouns = billions of combinations
- **Readable output**: No random strings, UUIDs, or hex—just words humans can remember and type
- **One implementation**: Rust core shared across all platforms via FFI and WASM
- **Zero config**: Works out of the box with sensible defaults

Use cases: container names, temporary credentials, session IDs, test fixtures, branch names, anything that needs a memorable identifier.

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
go get github.com/babbitt/harp/bindings/go
```

```go
import "github.com/babbitt/harp/bindings/go"

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

### CLI

```bash
cargo install harp-cli
```

```bash
$ harp
swift-amber-falcon
```

## Options

All bindings support the same configuration:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `components` | 2-16 | 3 | Number of words (N-1 adjectives + 1 noun) |
| `max_element_length` | uint | none | Maximum characters per word |
| `separator` | string | `"-"` | Delimiter between words |

### Rust

```rust
use harp_core::{generate_name_with_options, NameOptions};

let opts = NameOptions {
    components: 2,
    max_element_length: Some(5),
    separator: "_".to_string(),
};
let name = generate_name_with_options(&opts);  // "bold_hawk"
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
name := harp.GenerateNameWithOptions(opts)  // "bold_hawk"
```

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
