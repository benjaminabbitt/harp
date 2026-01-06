# Harp

**H**uman **A**ppropriate **R**andom **P**hraselets

Random name generator that creates human-readable names by combining adjectives and nouns.

*Name credit: [AdamPIcode](https://old.reddit.com/user/AdamPIcode)*

```
misty-golden-river
swift-amber-falcon
quiet-silver-meadow
```

## Features

- 17,874 adjectives and 55,191 nouns
- Configurable: 2-16 components, custom separators, max element length
- Multi-language: Rust, Python, Go, JavaScript/TypeScript, C/C++

## Installation

### Rust

```toml
[dependencies]
harp-core = "0.1"
```

### CLI

```bash
cargo install harp-cli
```

### Python

```bash
pip install harp-names
```

### Go

```bash
go get github.com/babbitt/harp/bindings/go
```

### JavaScript/TypeScript

```bash
npm install @babbitt/harp-wasm
```

## Usage

### Rust

```rust
use harp_core::{generate_name, generate_name_with_options, NameOptions};

// Default: 2 adjectives + 1 noun
let name = generate_name();
// => "misty-golden-river"

// Custom options
let opts = NameOptions {
    components: 2,
    max_element_length: Some(5),
    separator: "_".to_string(),
};
let name = generate_name_with_options(&opts);
// => "swift_hawk"
```

### CLI

```bash
harp
# => misty-golden-river
```

### Python

```python
import harp

name = harp.generate_name()
# => "misty-golden-river"

name = harp.generate_name_with_options(components=2, separator="_")
# => "swift_hawk"
```

### Go

```go
import "github.com/babbitt/harp/bindings/go"

name := harp.GenerateName()
// => "misty-golden-river"

name = harp.GenerateNameWithOptions(2, 0, "_")
// => "swift_hawk"
```

### JavaScript/TypeScript

```javascript
import init, { generate_name } from '@babbitt/harp-wasm';

await init();
const name = generate_name();
// => "misty-golden-river"
```

## Building

Requires [just](https://github.com/casey/just) command runner.

```bash
# Build all Rust crates
just build

# Run tests
just test

# Build Python bindings (requires maturin)
just build-python

# Build WASM bindings (requires wasm-pack)
just build-wasm

# Build Go bindings
just build-wasm-core
just test-go

# See all commands
just --list
```

### Docker

```bash
# Build everything in Docker
just docker-all

# Individual builds
just docker-rust
just docker-python
just docker-go
just docker-wasm
```

## Architecture

```
crates/
├── harp-core        # Core library
├── harp-cli         # Command-line interface
├── harp-ffi         # C FFI bindings
├── harp-python      # Python bindings (PyO3)
├── harp-wasm        # Browser/Node.js WASM
└── harp-wasm-core   # WASI WASM (for Go)

bindings/
├── go/              # Go package
├── typescript/      # npm package
└── python/          # PyPI package
```

## License

BSD-3-Clause
