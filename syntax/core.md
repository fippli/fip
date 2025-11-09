# Core Functions

This page documents the small standard library that ships with the runtime. All functions are curried: supplying fewer arguments than listed returns a new function waiting for the rest.

## log!

**Signature** `log!: (message) -> null`

**Behavior** Writes `message` to standard output. Returns `null`. Must be marked impure (`!`) because it performs IO. Never throws.

**Example**

```
log!("hello, world")
```

## trace!

**Signature** `trace!: (label, value) -> value`

**Behavior** Prints `(trace) <label>: <value>` to standard output and returns `value` unchanged so it can stay in a pipeline. Impure due to IO.

**Example**

```
trace!("name", "Filip") // prints "(trace) name: Filip" and yields "Filip"
```

## identity

**Signature** `identity: (x) -> x`

**Behavior** Returns the argument without modification.

**Example**

```
identity("hello") // "hello"
```

## increment

**Signature** `increment: (number) -> number`

**Behavior** Adds one to the numeric argument. Errors if the argument is not numeric.

**Example**

```
increment(3) // 4
```

## decrement

**Signature** `decrement: (number) -> number`

**Behavior** Subtracts one from the numeric argument. Errors if the argument is not numeric.

**Example**

```
decrement(3) // 2
```

## map

**Signature** `map: (fn, list) -> list`

**Behavior** Produces a new list by invoking `fn` on each element of `list` from left to right. `fn` receives the current element and must return the transformed value. The input list is never mutated.

**Example**

```
numbers: [1, 2, 3]
map(increment, numbers) // [2, 3, 4]
```

## reduce

**Signature** `reduce: (fn, init, list) -> value`

**Behavior** Folds `list` into a single value. `fn` is called with `(accumulator, element)` for each element. The first call uses `init` as the accumulator. The last accumulator returned by `fn` becomes the result. Works on empty lists by immediately returning `init`.

**Example**

```
numbers: [1, 2, 3]
reduce((acc, n) { acc + n }, 0, numbers) // 6
```

## filter

**Signature** `filter: (predicate, list) -> list`

**Behavior** Returns a new list containing only the elements for which `predicate(element)` returns `true`. Evaluation keeps the original order.

**Example**

```
numbers: [1, 2, 3]
is-two?: (n) { n = 2 }
filter(is-two?, numbers) // [2]
```

## every?

**Signature** `every?: (predicate, list) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `true` for every element of `list`. Returns `true` for an empty list. Stops early on the first `false`.

**Example**

```
numbers: [2, 4, 6]
is-even?: (n) { n % 2 = 0 }
every?(is-even?, numbers) // true
```

## some?

**Signature** `some?: (predicate, list) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `true` for at least one element of `list`. Returns `false` for an empty list. Stops early on the first `true`.

**Example**

```
numbers: [1, 2, 3]
some?((n){ n = 2 }, numbers) // true
```

## none?

**Signature** `none?: (predicate, list) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `false` for every element of `list`. Equivalent to `not(every?(predicate, list))`. Returns `true` for an empty list.

**Example**

```
numbers: [1, 3, 5]
none?((n){ n % 2 = 0 }, numbers) // true
```

## defined?

**Signature** `defined?: (value) -> boolean`

**Behavior** Returns `false` when `value` is `null`, otherwise `true`. Commonly used to guard optional data.

**Example**

```
defined?(null) // false
defined?(123) // true
```

## if

**Signature** `if: (condition, then-fn, else-fn) -> value`

**Behavior** `condition` must be a boolean. `then-fn` and `else-fn` are zero-argument functions (thunks). Only one branch is invoked: `then-fn()` when the condition is `true`, otherwise `else-fn()`. The return value of the invoked branch becomes the result.

**Example**

```
result-true: if(true, () { "true" }, () { "false" })
// "true"

maybe-value: 12345
safe: if(defined?(maybe-value), () { maybe-value }, () { "No value" })
// 12345

missing: null
fallback: if(defined?(missing), () { missing }, () { "No value" })
// "No value"
```

## for-each!

**Signature** `for-each!: (fn, list) -> null`

**Behavior** Iterates through `list` from left to right, invoking `fn` for each element. `fn` must be an impure single-argument function—any value it returns is ignored. `for-each!` itself is impure because it sequences side effects and always returns `null`.

**Example**

```
words: ["a", "b", "c"]
for-each!((word)! { log!(word) }, words)
// prints each word on its own line
```
