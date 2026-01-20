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
    components=2,              # 2-16 words
    max_element_length=5,      # max chars per word (None = no limit)
    separator="_"              # custom separator
)
print(name)  # e.g., "bold_hawk"
```

## API

- `generate_name()` - Generate a random name (2 adjectives + 1 noun)
- `generate_name_with_options(components=3, max_element_length=None, separator="-")` - Generate with options
- `version()` - Get the library version

## License

BSD-3-Clause
