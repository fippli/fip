# FIP Linter

A static analysis tool for FIP (Functional Intuitive Programming language) that checks code for common errors and style violations.

## Features

- **Function Notation Checks**:
  - Detects functions marked as impure (`!`) that don't call impure functions
  - Detects functions calling impure functions without the `!` suffix
  - Detects boolean functions (`?`) that don't return boolean values

- **Code Quality**:
  - Validates function purity rules
  - Checks anonymous function notation

## Installation

Build the linter:

```bash
cd tools/linter
cargo build --release
```

The binary will be available at `target/release/fip-lint`.

## Usage

```bash
# Lint a single file
fip-lint path/to/file.fip

# Exit code 0 = no errors, 1 = errors found
```

## Lint Rules

### Impure Function Notation

Functions that call impure functions (ending with `!`) must be marked as impure:

```fip
// ❌ Error: Function must be declared impure
foo: (x) {
  log!(x)
}

// ✅ Correct
foo!: (x) {
  log!(x)
}
```

Functions marked as impure must actually call impure functions:

```fip
// ❌ Error: Function is marked impure but performs no impure operations
bar!: (x) {
  x + 1
}

// ✅ Correct
bar!: (x) {
  log!(x)
  x + 1
}
```

### Boolean Function Notation

Functions ending with `?` must return boolean values:

```fip
// ❌ Error: Function must return a boolean value
is-it?: (x) {
  x + 3
}

// ✅ Correct
is-zero?: (x) {
  x = 0
}
```

### Anonymous Functions

Anonymous functions follow the same rules:

```fip
// ❌ Error: Anonymous function must be marked impure
map((n) { log!(n) }, numbers)

// ✅ Correct
map((n)! { log!(n) }, numbers)
```

## Integration

### VS Code/Cursor Extension

See `../linter-extension` for the VS Code extension that integrates this linter.

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
find . -name "*.fip" -exec fip-lint {} \;
```

### CI/CD

```yaml
# Example GitHub Actions
- name: Lint FIP files
  run: |
    find . -name "*.fip" -exec fip-lint {} \;
```

## Output Format

The linter outputs errors in the following format:

```
file.fip:line:column: severity: message
```

Example:

```
test.fip:5:1: error: Function 'foo' must be declared impure (end the name with '!') to call 'log!'
test.fip:10:1: error: Function 'is-it?' must return a boolean value
```

## Exit Codes

- `0`: No linting errors found
- `1`: Linting errors found

## License

MIT

