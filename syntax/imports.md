# Imports

Fippli programs can pull definitions from other files using the `use` statement. This enables code splitting across multiple `.fip` modules while keeping the call sites explicit.

## Basic syntax

**Signature** `use <name> from "<module-path>"`

**Behavior** Binds the exported value `name` from the referenced module path into the current scope. Module paths resolve relative to the program entry point directory (for example, `src`), not relative to the importing file.

**Example**

```fip
use foo from "lib/foo"
log!(foo())
// -> null
```

## Semantics

- Each imported file is evaluated once. Subsequent `use` statements for the same module path reuse the previously computed module environment.
- A module must explicitly declare which bindings it exports. Importing a module binds only the exported value associated with the requested name.
- Import cycles are detected at runtime; attempting to load modules that depend on each other produces a descriptive error.

## Namespace imports

**Signature** `use <module> as <alias> from "<module-path>"`

**Behavior** Imports the entire module environment under an alias. Access individual bindings with property notation (`alias.increment`).

**Example**

```fip
use math as m from "core/math"

m.increment(41)
// -> 42
```

## Selective imports

**Signature** `use { name-1, name-2 } from "<module-path>"`

**Behavior** Imports multiple named exports from the same module. Each listed name must be exported by the target module; missing exports raise a runtime error naming the missing identifier and module path.

**Example**

```fip
use {increment, decrement} from "core/math"

increment(1)
// -> 2

decrement(1)
// -> 0
```

## Error handling

- The module file cannot be found at the resolved path.
- The module fails to evaluate.
- The requested binding is not exported by the module.
- An import cycle is encountered.

The interpreter raises a runtime error in these cases and includes the original `use` site plus the attempted module path to aid debugging.
