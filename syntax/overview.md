# Overview

FIP programs are sequences of expressions evaluated from top to bottom. Each binding introduces a new name, every expression produces a value, and functions are curried by default. This overview walks through the essential pieces you need before digging into the detailed specification sections.

## Basic structure

- Files use the `.fip` extension.
- Newline-separated expressions run in order; the value from one line can feed into the next when used inside composable blocks.
- Bindings use the `name: expression` syntax and are immutable for the lifetime of their scope.

```fip
// hello.fip
name: "Filip"
message: "Hello, <name>!"
log!(message)
```

## Evaluation model

1. Bindings evaluate lazily within composable blocks, otherwise eagerly on assignment.
2. Attempting to reassign an existing name in the same scope is a compile-time error.
3. All functions return the value of their final expression; no implicit `return` keyword exists.

## Functions and currying

Functions follow `fn-name: (arg1, arg2) { body }`. Multiple parameters are syntactic sugar for nested single-argument functions, so partial application works everywhere.

```fip
add: (x, y) { x + y }
add-one: add(1)
add-one(4) // 5
```

Anonymous functions drop the name but keep the same parameter and body structure: `(x) { x + 1 }`.

## Purity markers

- Append `!` to a function name or parameter list when it performs side effects (`log!`, `for-each!`). Impure functions can only be called from other impure functions.
- Append `?` when a function returns a boolean (`defined?`, `is-zero?`). Marking a non-boolean function with `?` is a compile-time error.

## Comments

Single-line comments start with `//` and continue to the end of the line. There is no block comment syntax yet.

```fip
// Print each number after doubling it
numbers: [1, 2, 3]
for-each!((n)! {
  doubled: n * 2
  log!(doubled)
}, numbers)
```

## Modules and imports

Use `use` statements to pull definitions from other files. Relative imports resolve from the current file's directory, and names are immutable just like local bindings.

```fip
use math.add from "./lib/math"
result: add(2, 3)
```

## Putting it together

A typical program weaves bindings, function calls, and composable blocks:

```fip
// Calculate the total cost with tax and log the steps
tax-rate: 0.25
items: [10, 20, 40]

subtotal: items
  reduce((total, price) { total + price }, 0)

total: subtotal * (1 + tax-rate)
trace!("subtotal", subtotal)
trace!("total", total)
```

