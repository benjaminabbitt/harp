# harp-go

Go bindings for harp - generate random names from adjectives and nouns.

## Architecture

This package uses Rust as the single source of truth via WebAssembly (WASM). The Rust `harp-core` library is compiled to WASM and embedded in this Go module, executed via [wazero](https://github.com/tetratelabs/wazero).

**Trade-offs:**
- **DRY**: All name generation logic lives in Rust. No reimplementation across languages.
- **Consistency**: Identical behavior guaranteed across all language bindings.
- **Size**: ~375KB embedded WASM binary (includes compressed word lists).
- **Runtime**: WASM interpreter overhead vs native code. Acceptable for most use cases.

This design prioritizes correctness and maintainability over optimal performance.

## Installation

```bash
go get github.com/benjaminabbitt/harp/bindings/go
```

No native compilation required - the WASM binary is embedded.

## Usage

```go
package main

import (
    "fmt"
    harp "github.com/benjaminabbitt/harp/bindings/go"
)

func main() {
    name := harp.GenerateName()
    fmt.Println(name) // e.g., "misty-golden-river"

    // With options
    opts := harp.Options{
        Components:       4,
        MaxElementLength: 6,
        Separator:        "_",
    }
    name = harp.GenerateNameWithOptions(opts)
    fmt.Println(name) // e.g., "calm_warm_soft_oak"
}
```

## API

- `GenerateName()` - Generate a random name (2 adjectives + 1 noun)
- `GenerateNameWithOptions(opts)` - Generate with custom options
- `Version()` - Get the library version
- `DefaultOptions()` - Get default options

## License

BSD-3-Clause
