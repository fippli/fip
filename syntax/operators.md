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
