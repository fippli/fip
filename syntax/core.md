# Core Functions

The core module provides a compact standard library that complements the language syntax. Each function is curried: calling it with fewer arguments than listed returns a new function waiting for the remainder. Use the pages under `core/` for detailed behavior, signatures, and examples.

## Reference Guides

- [Identity](core/identity.md) — Identity helpers such as `identity`.
- [Values](core/values.md) — Single-value utilities like `.map` and `defined?`.
- [Math](core/math.md) — Numeric utilities including `sum`, `add`, `subtract`, `multiply`, `divide`, `divide-by`, `increment`, and `decrement`.
- [Array Helpers](core/array.md) — Collection routines like `map`, `reduce`, and predicates.
- [Effects](core/effects.md) — Impure helpers like `log!`, `trace!`, and `for-each!`.
- [Control](core/control.md) — Conditional helpers such as the callable `if`.
- [Object Helpers](core/object.md) — Reserved for record utilities; update this page as new functions land.
