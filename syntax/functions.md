# Functions

Functions return the value produced by the final expression in their body. Every function is immutable and curried by default, which means partial application always yields another callable that captures the provided arguments.

## Definition

**Signature** `<name>: (<parameters>) { <body> }`

**Behavior** Declares an immutable function binding. Parameters are evaluated when the function is called, not when it is defined. The final expression inside the body becomes the return value.

**Example**

```fip
identity: (x) { x }
// -> <function>

identity(42)
// -> 42
```

### Multiple arguments

**Signature** `<name>: (param-1, param-2, ...) { <body> }`

**Behavior** Listing multiple parameters is sugar for nesting single-argument functions. The runtime still curries them, so you can call the function with any prefix of arguments.

**Example**

```fip
add: (x, y) { x + y }
// -> <function>

add(1, 2)
// -> 3
```

### Currying

**Signature** `fn(arg-1, arg-2, ...)`

**Behavior** Calling a function with fewer arguments than declared returns a new function that expects the remaining arguments. Supplying all arguments at once works because the interpreter applies them from left to right.

**Example**

```fip
add3: (x, y, z) { x + y + z }
// -> <function>

add1: add3(1)
// -> <function awaiting y, z>

add1-and-2: add1(2)
// -> <function awaiting z>

add1-and-2(3)
// -> 6
```

## Function call

**Signature** `fn(arg-1, arg-2, ...) -> value`

**Behavior** Evaluates the callee and each argument, then applies them. Curried results can be called immediately or stored for later use.

**Example**

```fip
increment: (x) { x + 1 }
// -> <function>

increment(5)
// -> 6

add: (x, y) { x + y }
// -> <function>

curried: add(10)
// -> <function awaiting y>

curried(7)
// -> 17
```

## Anonymous functions

**Signature** `(params) { <body> }`

**Behavior** Defines a function without a name. Anonymous functions obey the same currying and purity rules as named functions and are commonly passed inline to higher-order helpers.

**Example**

```fip
(x) { x * 2 }(3)
// -> 6
```

### Purity suffixes

**Signature** `(params)! { <body> } | (params)? { <body> }`

**Behavior** Attach `!` to mark an anonymous function as impure, or `?` when it returns a boolean. The interpreter enforces the same suffix rules as for named functions.

**Example**

```fip
numbers: [1, -1, 2]

filter((n)? { n > 0 }, numbers)
// -> [1, 2]

map((n)! {
  trace!("doubling", n)
  n + n
}, numbers)
// -> [2, -2, 4]
```

## Composable blocks

**Signature** `{ expression-1; expression-2; ... }`

**Behavior** Compose operations by stacking expressions—each line feeds the next. This style works well with functions that accept a single argument.

**Example**

```fip
pipeline: (value) {
  value
  increment
  increment
  identity
}
// -> <function>

pipeline(1)
// -> 3
```

## Function notations

### Impure notation `!`

**Signature** `<name>!: (params) { <body> }`

**Behavior** Append `!` to indicate that a function performs side effects (logging, tracing, IO, etc.). If a function calls any impure helper, it must also use the `!` suffix. The runtime rejects functions marked with `!` when no impure calls occur in the body.

**Example**

```fip
logger!: (message) { log!(message) }
// -> <function>

logger!("hello")
// -> null
```

### Boolean notation `?`

**Signature** `<name>?: (params) { <body> }`

**Behavior** Use the `?` suffix for functions that return a boolean value. A function that does not return a boolean may not use the `?` suffix.

**Example**

```fip
is-zero?: (x) { x = 0 }
// -> <function>

is-zero?(0)
// -> true
```
