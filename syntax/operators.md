# Operators

Operators manipulate primitive values, combine expressions, and control evaluation flow. All operators are immutable: they return new values without mutating existing bindings. This guide documents each operator’s signature, behavior, and canonical example.

## Assignment `:`

**Signature** `name: expression -> value`

**Behavior** Binds the evaluated expression to `name` in the current scope. Rebinding the same name in the same scope is illegal; use a new name when deriving additional values. Returns the bound value, enabling pipelines inside composable blocks.

**Example**

```fip
n: 123
// -> 123

name: "Filip"
// -> "Filip"

identity: (x) { x }
// -> <function>
```

## Addition `+`

**Signature** `number + number -> number`

**Behavior** Adds two integers and returns their sum. Both operands must be numbers; otherwise the runtime raises a “Doesn't make sense” error.

**Example**

```fip
1 + 2
// -> 3
```

## Subtraction `-`

**Signature** `number - number -> number`

**Behavior** Subtracts the right operand from the left. Operands must be numbers.

**Example**

```fip
5 - 2
// -> 3
```

## Multiplication `*`

**Signature** `number * number -> number`

**Behavior** Multiplies two numbers. Inputs must be numeric.

**Example**

```fip
2 * 3
// -> 6
```

## Division `/`

**Signature** `number / number -> number`

**Behavior** Divides the left operand by the right. Both operands must be numbers. Division by zero raises a runtime error.

**Example**

```fip
8 / 2
// -> 4
```

## Equality `=`

**Signature** `value = value -> boolean`

**Behavior** Compares two values structurally. Returns `true` when both operands share the same type and value.

**Example**

```fip
"hello" = "hello"
// -> true
```

## Not equal `≠`

**Signature** `value ≠ value -> boolean`

**Behavior** Negates structural equality. Returns `true` when operands differ.

**Example**

```fip
1 ≠ 2
// -> true
```

## Less than `<`

**Signature** `number < number -> boolean`

**Behavior** Returns `true` when the left operand is strictly smaller than the right operand.

**Example**

```fip
3 < 5
// -> true
```

## Greater than `>`

**Signature** `number > number -> boolean`

**Behavior** Returns `true` when the left operand is strictly larger than the right operand.

**Example**

```fip
10 > 1
// -> true
```

## Less than or equal `<=`

**Signature** `number <= number -> boolean`

**Behavior** Returns `true` when the left operand is smaller than or equal to the right operand.

**Example**

```fip
3 <= 3
// -> true
```

## Greater than or equal `>=`

**Signature** `number >= number -> boolean`

**Behavior** Returns `true` when the left operand is larger than or equal to the right operand.

**Example**

```fip
1 >= 10
// -> false
```

## Logical and `&`

**Signature** `boolean & boolean -> boolean`

**Behavior** Evaluates both operands and returns `true` only when both are `true`. Inputs must be boolean values.

**Example**

```fip
true & false
// -> false
```

## Logical or `|`

**Signature** `boolean | boolean -> boolean`

**Behavior** Returns `true` when either operand is `true`. Both operands must be booleans.

**Example**

```fip
false | true
// -> true
```

## Spread `...`

### Object spread

**Signature** `{ ...object, key: value } -> object`

**Behavior** Copies fields from one object into a new object. Later fields overwrite earlier ones when keys collide. Source objects remain unchanged.

**Example**

```fip
base: { name: "Jim" }
// -> { name: "Jim" }

with-age: { ...base, age: 100 }
// -> { name: "Jim", age: 100 }

updated: { ...with-age, age: 75 }
// -> { name: "Jim", age: 75 }
```

### Array spread

**Signature** `[...array, value, ...] -> array`

**Behavior** Expands array elements into a new array literal. The result is a fresh array; the original is unchanged.

**Example**

```fip
a: [1, 2, 3]
// -> [1, 2, 3]

b: [...a, 4, 5]
// -> [1, 2, 3, 4, 5]

c: [0, ...b]
// -> [0, 1, 2, 3, 4, 5]
```
