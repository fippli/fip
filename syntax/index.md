# FIP (Functional Intuitive Programming language)

FIP is a small, expression-oriented language focused on immutable data and composable functions. Programs read top-to-bottom, with each binding introducing a new name and every function call returning a value that can flow into the next expression. The syntax emphasizes clarity, currying by default, and explicit purity markers so effects remain predictable.

- File extension `.fip`
- Pure-by-default functional style
- Explicit annotations for impure (`!`) and boolean (`?`) helpers

## Specification

1. `./overview.md` — quick tour of basic syntax and evaluation rules.
2. `./variables.md` — name binding, scope, and immutability.
3. `./data-types.md` — numbers, strings, booleans, lists, objects, and null.
4. `./operators.md` — arithmetic, comparison, logical, and chaining operators.
5. `./functions.md` — function definitions, currying, anonymous forms, and composition.
6. `./core.md` — standard library helpers such as `log!`, `map`, and `if`.
7. `./data-structures.md` — constructing and manipulating structured data.
8. `./imports.md` — module system and reuse of code across files.
9. `./errors.md` — runtime and compile-time error semantics.
10. `./async.md` — planned async primitives like `async`/`await` (experimental).
11. `./test.md` — guidelines for writing tests in FIP (in progress).
