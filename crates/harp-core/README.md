# harp-core

Core library for generating random names from adjectives and nouns.

## Usage

```rust
use harp_core::generate_name;

fn main() {
    let name = generate_name();
    println!("{}", name); // e.g., "misty-golden-river"
}
```

## API

- `generate_name()` - Generate a random name from two adjectives and a noun
- `ADJECTIVES` - List of available adjectives (17,874 words)
- `NOUNS` - List of available nouns (55,191 words)

## License

MIT
