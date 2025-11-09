# FIP Language Syntax Highlighting for VS Code

Syntax highlighting extension for FIP (Functional Intuitive Programming language).

## Features

- Syntax highlighting for FIP language files (`.fip`)
- Support for:
  - Function definitions (including impure `!` and predicate `?` notation)
  - Strings with interpolation (`<name>`)
  - Numbers
  - Objects and lists
  - Operators (`+`, `-`, `*`, `/`, `=`, `&`, `|`)
  - Comments (`//`)
  - Keywords (`null`, `true`, `false`)

## Installation

### For Cursor (Recommended)

**Option 1: Install from Folder (Easiest)**
1. Open Cursor
2. Press `F1` or `Cmd+Shift+P` (Mac) / `Ctrl+Shift+P` (Windows/Linux)
3. Type "Extensions: Install from Folder..." and select it
4. Navigate to and select the `tools/syntax` directory
5. Reload Cursor

**Option 2: Manual Installation**
1. Copy the `tools/syntax` directory to your Cursor extensions folder:
   - **Windows**: `%USERPROFILE%\.cursor\extensions\`
   - **macOS**: `~/.cursor/extensions/`
   - **Linux**: `~/.cursor/extensions/`
2. Rename it to `fip-syntax`
3. Reload Cursor

**Option 3: Development Symlink (for active development)**
```bash
# On macOS/Linux
ln -s /Users/filipjohansson/dev/fippli/fip/tools/syntax ~/.cursor/extensions/fip-syntax

# On Windows (PowerShell)
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.cursor\extensions\fip-syntax" -Target "C:\path\to\fip\tools\syntax"
```

Then reload Cursor.

### For VS Code

**Option 1: Install from Folder**
1. Open VS Code
2. Press `F1` or `Cmd+Shift+P` (Mac) / `Ctrl+Shift+P` (Windows/Linux)
3. Type "Extensions: Install from Folder..." and select it
4. Navigate to and select the `tools/syntax` directory
5. Reload VS Code

**Option 2: Manual Installation**
1. Copy the `tools/syntax` directory to your VS Code extensions folder:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\`
   - **macOS**: `~/.vscode/extensions/`
   - **Linux**: `~/.vscode/extensions/`
2. Rename it to `fip-syntax`
3. Reload VS Code

## Usage

Open any `.fip` file in Cursor or VS Code and enjoy syntax highlighting!

## Language Features

- **File extension**: `.fip`
- **Comments**: `//` for single-line comments
- **Strings**: Double quotes with interpolation support (`"Hello <name>"`)
- **Functions**: `name: (params) { body }`
- **Impure functions**: `name!: (params) { body }`
- **Predicate functions**: `name?: (params) { body }`
- **Objects**: `{ key: value }`
- **Lists**: `[1, 2, 3]`

## Contributing

Feel free to submit issues or pull requests to improve the syntax highlighting.

## License

MIT
