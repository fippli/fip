# Core Array Helpers

Array helpers operate on ordered collections (`[ ... ]`). Unless otherwise noted they never mutate their inputs and return new arrays. See [data-types](../data-types.md) for collection semantics and [functions](../functions.md) for details on currying.

## map

**Signature** `map: (fn, array) -> array`

**Behavior** Produces a new array by invoking `fn` on each element of `array` from left to right. `fn` receives the current element and must return the transformed value. The original array is left untouched.

**Example**

```fip
numbers: [1, 2, 3]

map(increment, numbers)
// -> [2, 3, 4]
```

## reduce

**Signature** `reduce: (fn, init, array) -> value`

**Behavior** Folds `array` into a single value. `fn` is called with `(accumulator, element)` for each element, starting with `init` as the first accumulator. The last accumulator returned by `fn` becomes the result and empty arrays immediately return `init`.

**Example**

```fip
numbers: [1, 2, 3]

reduce((acc, n) { acc + n }, 0, numbers)
// -> 6
```

## filter

**Signature** `filter: (predicate, array) -> array`

**Behavior** Returns a new array containing only the elements for which `predicate(element)` returns `true`. Preserves the original order.

**Example**

```fip
numbers: [1, 2, 3]

is-two?: (n) { n = 2 }
// -> <function>

filter(is-two?, numbers)
// -> [2]
```

## every?

**Signature** `every?: (predicate, array) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `true` for every element of `array`. Returns `true` for an empty array and stops early on the first `false`.

**Example**

```fip
numbers: [2, 4, 6]

is-even?: (n) { n % 2 = 0 }
// -> <function>

every?(is-even?, numbers)
// -> true
```

## some?

**Signature** `some?: (predicate, array) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `true` for at least one element of `array`. Returns `false` for an empty array and stops early on the first `true`.

**Example**

```fip
numbers: [1, 2, 3]

some?((n) { n = 2 }, numbers)
// -> true
```

## none?

**Signature** `none?: (predicate, array) -> boolean`

**Behavior** Returns `true` if `predicate(element)` is `false` for every element of `array`. Equivalent to `not(every?(predicate, array))` and returns `true` for an empty array.

**Example**

```fip
numbers: [1, 3, 5]

none?((n) { n % 2 = 0 }, numbers)
// -> true
```
