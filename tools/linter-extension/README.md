# FIP Linter Extension

VS Code/Cursor extension that provides linting for FIP (Functional Intuitive Programming language) files.

## Features

- Real-time linting of FIP files
- Highlights errors, warnings, and info messages
- Checks for:
  - Function notation violations (impure `!` and boolean `?`)
  - Missing impure markers on functions calling impure functions
  - Boolean functions that don't return boolean values
  - Other code quality issues

## Installation

### Prerequisites

1. Build the linter binary:
   ```bash
   cd tools/linter
   cargo build --release
   ```

2. Make sure `fip-lint` is in your PATH, or configure the path in VS Code settings.

### Install Extension

1. Open VS Code/Cursor
2. Press `F1` or `Cmd+Shift+P` (Mac) / `Ctrl+Shift+P` (Windows/Linux)
3. Type "Extensions: Install from Folder..."
4. Select the `tools/linter-extension` directory
5. Reload VS Code/Cursor

### Development Installation

For development, create a symlink:

```bash
# On macOS/Linux
ln -s /path/to/fip/tools/linter-extension ~/.cursor/extensions/fip-linter

# On Windows (PowerShell)
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.cursor\extensions\fip-linter" -Target "C:\path\to\fip\tools\linter-extension"
```

Then:
1. Install dependencies: `cd tools/linter-extension && npm install`
2. Compile: `npm run compile`
3. Reload VS Code/Cursor

## Configuration

Add to your VS Code settings (`.vscode/settings.json` or user settings):

```json
{
  "fipLinter.enable": true,
  "fipLinter.path": "/path/to/fip-lint"
}
```

If `fip-lint` is in your PATH, you can just use:

```json
{
  "fipLinter.enable": true,
  "fipLinter.path": "fip-lint"
}
```

## Usage

The extension automatically lints FIP files when you:
- Open a `.fip` file
- Save a `.fip` file
- Make changes to a `.fip` file (with a 500ms debounce)

Linting errors will appear as red squiggles, warnings as yellow, and info messages as blue.

## Building

```bash
cd tools/linter-extension
npm install
npm run compile
```

## License

MIT

