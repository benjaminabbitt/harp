# @babbitt/harp-wasm

Generate random names from adjectives and nouns (WebAssembly).

## Installation

```bash
npm install @babbitt/harp-wasm
```

## Usage

```javascript
import init, { generate_name } from '@babbitt/harp-wasm';

await init();
const name = generate_name();
console.log(name); // e.g., "misty-golden-river"
```

## API

- `generate_name()` - Generate a random name from two adjectives and a noun
- `version()` - Get the library version

## License

Licensed under either of Apache License 2.0 or BSD 3-Clause, at your option.
