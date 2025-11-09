# Operators

## Assignment `:`

Assign values to names with `:`.

### Example

```
n: 123
name: "Filip"
fn: (x) { x }
x: { foo: "bar" }
```

## Addition `+`

Adds two numeric values. Operands must be numbers.

### Example

```
1 + 2 // -> 3
```

### add (builtin)

```
add: (x,y) { x + y }
```

## Subtraction `-`

Subtracts the right operand from the left operand. Operands must be numbers.

```
1 - 2 // -> -1
```

### subtract (builtin)

```
subtract: (x,y) { x - y }
```

## Division `/`

Divides the left operand by the right operand. Operands must be numbers.

```
1 / 2 // -> 1/2
```

### divide (builtin)

```
divide: (x,y) { x / y }
```

## Multiplication `*`

Multiplies the operands. Operands must be numbers.

```
2 * 3 // -> 6
```

### multiply (builtin)

```
multiply: (x, y) { x * y }
```

## And `&`

Logical conjunction. Returns `true` only when both operands are `true`.

```
true & true // true
false & false // false
false & true // false
true & false // false
```

### and? (builtin)

```
and?: (x, y) { x & y }
```

## Or `|`

Logical disjunction. Returns `true` when at least one operand is `true`.

```
true | true // true
false | false // false
false | true // true
true | false // true
```

### or? (builtin)

```
or?: (x,y) { x | y }
```

## Comparison `=`

Structural equality. Returns `true` when both operands are equal by value and type.

```
1 = 1 // true
1 = 2 // false
"hello" = "hello" // true
"hello" = "foo" // false
1 = true // false
23 = "hello" // false
```

## Not equal `≠`

The opposite of `=`. Returns `true` when the operands are not equal. Works on any comparable values (numbers, strings, booleans, lists, objects).

```
1 ≠ 2 // true
"hello" ≠ "hello" // false
{ name: "Filip" } ≠ { name: "Filip" } // false
```

## Less than `<`

Numeric comparison. Both operands must be numbers. Returns `true` when the left side is strictly smaller.

```
3 < 5 // true
10 < 1 // false
```

## Greater than `>`

Numeric comparison. Both operands must be numbers. Returns `true` when the left side is strictly larger.

```
3 > 5 // false
10 > 1 // true
```

## Less than or equal `<=`

Numeric comparison that allows equality. Both operands must be numbers.

```
3 <= 3 // true
1 <= 0 // false
```

## Greater than or equal `>=`

Numeric comparison that allows equality. Both operands must be numbers.

```
3 >= 3 // true
1 >= 10 // false
```

## Spread

Spread operator

```
x: { name: "Jim" }
y: { ...x, age: 100 } // { name: "Jim", age: 100 }
z: { ...y, age: 75 } // { name: "Jim", age: 75 }
```

Arrays

```
a: [1, 2, 3]
b: [...a, 4, 5] // [1, 2, 3, 4, 5]
c: [0, ...b] // [0, 1, 2, 3, 4, 5]
```
