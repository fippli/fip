# Core functions

## log!

`log!` writes a message to the console. It is impure and therefore requires the `!` suffix.

**Signature:** `log!: (message) -> null`

**Example**

```
log!("hello, world")
```

## trace!

`trace!` logs both a message and the inspected value, returning the original value unchanged.
The output format is `(trace) <message>: <value>`. Like `log!`, it is impure.

**Signature:** `trace!: (message, value) -> value`

**Example**

```
trace!("name", "Filip") // (trace) name: Filip
```

```
f: (x) {
  x
  trace!("number", x)
  increment
}

f(1) // logs "(trace) number: 1" and returns 2
```

## identity

Returns the input exactly as received.

**Signature:** `identity: (x) { x }`

**Example**

```
identity("hello") // "hello"
```

## increment

Adds one to a numeric argument.

**Signature:** `increment: (x) { x + 1 }`

**Example**

```
increment(3) // 4
```

## decrement

Subtracts one from a numeric argument.

**Signature:** `decrement: (x) { x - 1 }`

**Example**

```
decrement(3) // 2
```

## map

Applies a function to each item in a list and returns a new list with the transformed values.

**Signature:** `map: (fn, list) -> list`

`fn` receives a single list item and must return the mapped value.

**Example**

```
numbers: [1, 2, 3]
map(increment, numbers) // [2, 3, 4]
```

## reduce

Combines a list into a single value by iteratively applying an accumulator function.

**Signature:** `reduce: (fn, init, list) -> value`

`fn` receives `(accumulator, item)` and must return the next accumulator value.

**Example**

```
numbers: [1, 2, 3]
reduce((acc, n) { acc + n }, 0, numbers) // 6
```

## filter

Keeps the items in a list that satisfy a predicate function and returns a new list.

**Signature:** `filter: (predicate, list) -> list`

`predicate` receives the list item and must return `true` for items to keep.

**Example**

```
numbers: [1, 2, 3]
is-two?: (n) { n = 2 }
filter(is-two?, numbers) // [2]
```
