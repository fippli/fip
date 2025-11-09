# Imports

Fippli programs can pull definitions from other files using the `use` statement. This enables code splitting across multiple `.fip` modules while keeping the call sites explicit.

## Basic Syntax

```
use <name> from "<module-path>"
```

- `name` is the identifier that will be introduced into the current scope.
- `module-path` is a string literal pointing to another `.fip` file.
- The module path is resolved relative to the program entry point directory (for example, `src`). It is **not** resolved relative to the file that issues the `use` statement.

Example:

```
use foo from "lib/foo"

log!(foo())
```

The example above loads `src/lib/foo.fip`, evaluates it (if it has not already been loaded), and binds the exported value to `foo` in the current scope.

## Semantics

- Each imported file is evaluated once. Subsequent `use` statements for the same module path reuse the previously computed module environment.
- A module must explicitly declare which bindings it exports. Importing a module binds only the exported value associated with the requested name.
- Import cycles are detected at runtime; attempting to load modules that depend on each other produces a descriptive error.

## Namespace Imports

You can import an entire module namespace with the `as` clause:

```
use math as m from "core/math"

m.increment(41)
```

This binds the module environment to the local identifier `m`. Access individual bindings using the familiar object property syntax.

## Selective Imports

Import multiple names from the same module with a destructuring-style form:

```
use {increment, decrement} from "core/math"

increment(1)
decrement(1)
```

Selective imports bind each listed name into the current scope. Missing exports trigger a runtime error that names the missing identifier and module path.

## Error Handling

The interpreter raises a runtime error in the following cases:

- The module file cannot be found at the resolved path.
- The module fails to evaluate.
- The requested binding is not exported by the module.
- An import cycle is encountered.

These errors include the original `use` site and the attempted module path to aid debugging.

