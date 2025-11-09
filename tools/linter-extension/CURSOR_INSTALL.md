# FIP Linter Extension - Cursor Installation Guide

## Quick Installation (Already Done!)

The extension has been symlinked to your Cursor extensions folder. **Reload Cursor** to activate it:

- Press `Cmd+Shift+P` and type "Reload Window"
- Or restart Cursor

## Configuration

The extension needs to know where the `fip-lint` binary is located. You have two options:

### Option 1: Configure the path in Cursor settings

1. Press `Cmd+Shift+P` in Cursor
2. Type "Preferences: Open User Settings (JSON)"
3. Add these settings:

```json
{
  "fipLinter.enable": true,
  "fipLinter.path": "/Users/filipjohansson/dev/fippli/fip/tools/linter/target/release/fip-lint"
}
```

### Option 2: Add fip-lint to your PATH (Recommended)

Add this to your `~/.zshrc`:

```bash
export PATH="$HOME/dev/fippli/fip/tools/linter/target/release:$PATH"
```

Then reload your shell and use this simpler config:

```json
{
  "fipLinter.enable": true,
  "fipLinter.path": "fip-lint"
}
```

## Verify Installation

1. Open a `.fip` file in Cursor
2. The extension should automatically lint it
3. Errors will show as red squiggles, warnings as yellow

## Troubleshooting

If linting doesn't work:

1. Check that the extension is loaded:

   - Press `Cmd+Shift+P`
   - Type "Extensions: Show Installed Extensions"
   - Look for "FIP Language Linter"

2. Check the path is correct:

   - Open Cursor settings
   - Search for "fipLinter"
   - Verify the path points to the correct location

3. Check the Output panel:
   - Press `Cmd+Shift+U` to open Output
   - Select "FIP Linter" from the dropdown
   - Look for any error messages

## Usage

The linter automatically runs when you:

- Open a `.fip` file
- Save a `.fip` file
- Make changes to a `.fip` file (500ms debounce)

You'll see linting errors highlighted directly in your code!
