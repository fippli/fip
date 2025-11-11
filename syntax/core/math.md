# Core Math

The math helpers provide simple numeric transformations. All functions assume their arguments are numbers and rely on the runtime's numeric type semantics discussed in [data-types](../data-types.md).

## add

**Signature** `add: (lhs, rhs) -> number`

**Behavior** Returns the sum of `lhs` and `rhs`. Both inputs must be numeric. Supports partial application: supplying only `lhs` returns a function that awaits `rhs`.

**Example**

```fip
add(2, 3)
// -> 5

add-five: add(5)
// -> <function>

add-five(10)
// -> 15
```

## subtract

**Signature** `subtract: (lhs, rhs) -> number`

**Behavior** Returns the result of `lhs - rhs`. Both inputs must be numeric. Partial application yields a function that subtracts its argument from the captured `lhs`.

**Example**

```fip
subtract(7, 2)
// -> 5

decrease-from-ten: subtract(10)
// -> <function>

decrease-from-ten(3)
// -> 7
```

## multiply

**Signature** `multiply: (lhs, rhs) -> number`

**Behavior** Returns the product of `lhs` and `rhs`. Both arguments must be numeric. Works with partial application for creating reusable scalars.

**Example**

```fip
multiply(4, 3)
// -> 12

double: multiply(2)
// -> <function>

double(6)
// -> 12
```

## divide

**Signature** `divide: (lhs, rhs) -> number`

**Behavior** Returns the quotient of `lhs / rhs`. Both arguments must be numeric. Raises a runtime error if `rhs` is zero. Partial application is useful for building reciprocal helpers.

**Example**

```fip
divide(12, 3)
// -> 4

divide-from-hundred: divide(100)
// -> <function>

divide-from-hundred(4)
// -> 25
```

## divide-by

**Signature** `divide-by: (denominator, numerator) -> number`

**Behavior** Divides `numerator` by `denominator`, reversing the argument order compared to `divide`. Useful when piping a value that should become the numerator. Errors if `denominator` is zero.

**Example**

```fip
divide-by(2, 4)
// -> 2

halve: divide-by(2)
// -> <function>

halve(9)
// -> 4.5

result: (
  4
  divide-by(2)
)
// -> 2
```

## increment

**Signature** `increment: (number) -> number`

**Behavior** Adds one to the numeric argument. Errors if the argument is not numeric.

**Example**

```fip
increment(3)
// -> 4
```

## decrement

**Signature** `decrement: (number) -> number`

**Behavior** Subtracts one from the numeric argument. Errors if the argument is not numeric.

**Example**

```fip
decrement(3)
// -> 2
```
