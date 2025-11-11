# Variables

FIP bindings are immutable by design, keeping program flow predictable. When you bind a name to a value, that relationship stays fixed for the scope unless you intentionally shadow it in a narrower block. This section covers how to declare bindings and how to name them consistently.

## Bindings

**Signature** `name: expression -> value`

**Behavior** Evaluates `expression` and binds the result to `name`. Rebinding the same name in the same scope raises a compile-time error. Use new identifiers to represent derived values.

**Example**

```fip
count: 3
// -> 3

next-count: count + 1
// -> 4

count: 4
// -> error: cannot reassign 'count' in the same scope
```

## Symbols

**Signature** `<segment-1>-<segment-2>-...`

**Behavior** Symbol names must be lower-case kebab case. Hyphenated segments improve readability and align with standard library naming. Names ending with `!` or `?` follow the purity and predicate conventions respectively.

**Example**

```fip
user-name: "Filip"
// -> "Filip"

is-active?: (flag) { flag }
// -> <function>

trace-action!: (message) { log!(message) }
// -> <function>
```
