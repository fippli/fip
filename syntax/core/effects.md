# Core Effects

The core effects module exposes impure helpers that interact with the outside world or sequence evaluation. Each function carries a `!` suffix and must run in an impure context. For an overview of purity, see [errors](../errors.md) and [functions](../functions.md).

## log!

**Signature** `log!: (message) -> null`

**Behavior** Writes `message` to standard output and returns `null`. Use it for quick diagnostics or user feedback without altering program state. Never throws and ignores its return value in pipelines.

**Example**

```fip
log!("hello, world")
// -> null
```

## trace!

**Signature** `trace!: (label, value) -> value`

**Behavior** Prints `(trace) <label>: <value>` to standard output and returns `value` unchanged so it can stay in a pipeline. Impure because it performs IO but otherwise side-effect free.

**Example**

```fip
trace!("name", "Filip")
// -> "Filip"
```

## for-each!

**Signature** `for-each!: (fn, array) -> null`

**Behavior** Iterates through `array` from left to right, invoking the single-argument impure function `fn` for each element. Any value returned by `fn` is ignored. `for-each!` sequences side effects, always returns `null`, and propagates errors thrown by `fn`.

**Example**

```fip
words: ["a", "b", "c"]

for-each!((word)! { log!(word) }, words)
// -> null
```
