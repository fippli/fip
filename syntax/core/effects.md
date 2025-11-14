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

## wait!

**Signature** `wait!: (fn, milliseconds) -> value`

**Behavior** Calls the function `fn` after waiting for `milliseconds` milliseconds and returns its result. The delay blocks execution, so `wait!` should be used sparingly. The function receives no arguments and its return value becomes the result of `wait!`. The function may be pure or impure.

**Implementation Notes** Uses the host runtime's sleep mechanism to pause execution. The delay is approximate and may vary based on system scheduling.

**Example**

```fip
result: wait!(() { 42 }, 1000)
// -> 42 (after 1 second)
```

## repeat!

**Signature** `repeat!: (fn, milliseconds) -> null`

**Behavior** Repeatedly calls a function `fn` with a delay of `milliseconds` between each call. The function continues to run indefinitely until the program terminates or an error occurs. The function receives no arguments and its return value is ignored. Always returns `null`. The function may be pure or impure.

**Implementation Notes** Runs in a loop, calling the function and then sleeping for the specified duration. The delay is approximate and may vary based on system scheduling.

**Example**

```fip
repeat!(() { 42 }, 1000)
// -> null (calls function every second, return value ignored)

count: 0

repeat!(()! {
  count: count + 1
  log!(count)
}, 1000)
// -> null
// (prints 1, 2, 3, ... every second)
```
