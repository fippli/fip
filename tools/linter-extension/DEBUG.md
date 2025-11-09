# Debugging FIP Linter Extension

## Check if Extension is Loaded

1. Press `Cmd+Shift+P` in Cursor
2. Type "Extensions: Show Installed Extensions"
3. Look for "FIP Language Linter" - it should be listed

## Check Extension Output

1. Press `Cmd+Shift+U` to open Output panel
2. Click the dropdown in the Output panel
3. Select "FIP Linter" (if it appears) or "Log (Extension Host)"
4. Look for any error messages

## Verify Configuration

1. Press `Cmd+Shift+P` → "Preferences: Open User Settings (JSON)"
2. Check that you have:

```json
{
  "fipLinter.enable": true,
  "fipLinter.path": "/Users/filipjohansson/dev/fippli/fip/tools/linter/target/release/fip-lint"
}
```

## Test the Linter Binary Directly

Run this in terminal to verify the linter works:

```bash
/Users/filipjohansson/dev/fippli/fip/tools/linter/target/release/fip-lint test-program/sum.fip
```

## Reload Extension

1. Press `Cmd+Shift+P`
2. Type "Developer: Reload Window"
3. Or restart Cursor completely

## Check File Language

Make sure your `.fip` file is recognized as FIP:

1. Look at the bottom-right of Cursor
2. It should say "FIP" or "Plain Text"
3. If it says "Plain Text", click it and select "FIP"

## Manual Test

Try opening `test-program/sum.fip` - it should lint without errors.
