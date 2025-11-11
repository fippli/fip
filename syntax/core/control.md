# Core Control

Control helpers decide which branch of a computation should run. They complement the language-level `if` expression discussed in [syntax/errors.md](../errors.md) by documenting the callable form exposed by the core module.

## if

**Signature** `if: (condition, then-fn, else-fn) -> value`

**Behavior** Evaluates `condition`, which must be a boolean. Invokes `then-fn()` when `condition` is `true`, otherwise `else-fn()`. Only the chosen branch runs and its return value becomes the result. Both thunks must be pure or impure together with the surrounding context.

**Example**

```fip
result-true: if(true, () { "true" }, () { "false" })
// -> "true"

maybe-value: 12345
// -> 12345

safe: if(defined?(maybe-value), () { maybe-value }, () { "No value" })
// -> 12345

missing: null
// -> null

fallback: if(defined?(missing), () { missing }, () { "No value" })
// -> "No value"
```
