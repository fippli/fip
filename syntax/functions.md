# Functions

Functions return the value produced by the final expression in their body.
In multi-line bodies, each line receives the result of the previous one unless a new binding is introduced.

## Definition

Functions are defined with the shape `<name>: (<parameters>) { <body> }`.

```
fn: (x) { x } // identity function definition
```

### Multiple arguments

Separate multiple parameters with commas.

```
g: (x, y) { x + y } // add function definition
g(1, 2) // 3
```

### Currying

Functions are curried by default. Writing multiple parameters is sugar for returning nested single-argument functions.

```
add3: (x, y, z) { x + y + z }
add1: add3(1)      // function waiting for y and z
add2: add1(2)      // function waiting for z
add2(3)            // 6
```

Calling a function with fewer arguments than its definition returns a new function that expects the remaining arguments. Supplying all arguments in a single call still works as usual because the runtime applies them from left to right.

## Function call

Provide arguments inside parentheses after the function name.

```
y: 2
f: (x) { x + 1 }
f(y) // 3
```

```
increment: (x) { x + 1 }
n: increment(1)
log!(n) // 2
```

## Anonymous function

Anonymous functions drop the name but otherwise follow the same syntax.

```
(){ 1 = 1 } // returns true
```

### Anonymous suffixes

Inline functions can adopt the same `!` and `?` suffix rules by attaching the suffix directly to the parameter list.

```
(x)! { log!(x) }    // impure anonymous function
(x)? { x = 0 }      // boolean anonymous function
```

Combine this with composable blocks or higher-order calls:

```
numbers.filter((n)? { n > 0 })
numbers.map((n)! {
  trace!("doubling", n)
  n + n
})
```

## Composable

Compose operations by stacking expressions—each line feeds the next.

```
identity: (x) { x }
increment: (x) { x + 1 }

f: (x) {
  x // passed as parameter to increment
  increment // incremented value passed to next increment
  increment
  identity
}

f(1) // 3
```

## Function notations

### Impure notation `!`

Append `!` to indicate that a function has side effects (logging, tracing, IO, etc.).

```
imp!: (x) { log!("string") }
```

If a function calls any impure function, it must also use the `!` suffix.

Correct:

```
foo!: (x) { log!(x) }
```

Wrong:

```
foo: (x) { log!(x) }
```

Note: The runtime rejects functions marked with `!` when no impure calls occur in the body.

#### Example

Wrong:

```
my-imp!: () {
  add(1, 2)
}
```

Should throw a "Not impure" error.

Correct:

```
my-imp!: () {
  trace!("This is so dirty...", 42)
}
```

### Boolean notation `?`

Use the `?` suffix for functions that return a boolean value.

```
is-equal?: (x, y) { x = y }
```

A function that does not return a boolean value may not use the `?` suffix.

#### Example

Wrong:

```
is-it?: (x) { x + 3 }
```

Correct:

```
is-zero?: (x) { x = 0 }
```
