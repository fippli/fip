# Comments

Comments document intent without affecting evaluation. They are stripped by the lexer before parsing, so they never change runtime behavior. Use them to explain tricky logic or annotate pipelines for future readers.

## Single-line comments

**Signature** `// <text>`

**Behavior** Everything after `//` on the same line is ignored. Comments may appear on their own line or after an expression. There is no block comment syntax; prefer multiple single-line comments for longer notes.

**Example**

```fip
// This line does nothing
log!("visible output") // trailing comments work too
// -> null
```
