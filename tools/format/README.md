# FIP Formatter

A code formatter for FIP (Functional Intuitive Programming language) files that enforces a consistent code style.

## Features

- Consistent indentation (2 spaces)
- Proper spacing around operators
- Unified formatting for functions, objects, lists, and lambdas
- Compact formatting for simple expressions
- Multi-line formatting for complex structures
- Preserves comments (via parsing)

## Installation

Build the formatter:

```bash
cd tools/format
cargo build --release
```

The binary will be available at `target/release/fip-format`.

## Usage

### Format a file and print to stdout

```bash
fip-format path/to/file.fip
```

### Format a file in-place

```bash
fip-format path/to/file.fip --write
# or
fip-format path/to/file.fip -w
```

### Format multiple files

```bash
for file in *.fip; do
    fip-format "$file" --write
done
```

## Formatting Rules

- **Indentation**: 2 spaces
- **Functions**: Multi-line with proper indentation
- **Objects**: Multi-line with fields on separate lines
- **Lists**: Single line if short, multi-line if long
- **Lambdas**: Compact `(params) { body }` for simple expressions, multi-line for complex
- **Operators**: Spaces around binary operators (`+`, `-`, `*`, `/`, `=`, `&`, `|`)
- **Statements**: Blank lines between top-level statements

## Examples

### Before formatting:

```fip
composed!:(x){
x
increment
(value)!{trace!("after first increment",value)}
increment
}
```

### After formatting:

```fip
composed!: (x) {
  x
  increment
  (value)! { trace!("after first increment", value) }
  increment
}
```

## Integration

### VS Code / Cursor

Add to your `settings.json`:

```json
{
  "[fip]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "terminal"
  }
}
```

Or use a task:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Format FIP",
      "type": "shell",
      "command": "${workspaceFolder}/tools/format/target/release/fip-format",
      "args": ["${file}", "--write"],
      "problemMatcher": []
    }
  ]
}
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
find . -name "*.fip" -exec tools/format/target/release/fip-format {} --write \;
```

## License

MIT
