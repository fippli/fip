# Core Values

Value helpers operate on single data items rather than whole collections. They are designed for use in pipelines and guards where you want to transform or validate the current value. See [functions](../functions.md) for currying semantics and [data-types](../data-types.md) for runtime value rules.

## .map

**Signature** `.map: (fn, value) -> value`

**Behavior** Applies `fn` to the provided `value` and returns the result. This lets you treat a lone value as a one-item collection inside a pipeline without breaking fluent APIs.

**Implementation Notes**

- Accept a callable `fn` and any `value`.
- Invoke `fn(value)` in the current purity context; propagate errors and purity requirements.
- Ensure `fn` expects exactly one argument.
- Return the result of the invocation without modification.

**Example**

```fip
.map(increment, 1)
// -> 2

enrich: () {
  { name: "Filip", age: 35, city: "Oslo" }
  .map((person) {
    {
      ...person
      name: "<person.name> Johansson"
    }
  })
  .map((person) {
    {
      ...person
      age: person.age + 1
    }
  })
  .map((person) {
    {
      ...person
      city: "New <person.city>"
    }
  })
}
// -> <function>

enrich()
// -> { name: "Filip Johansson", age: 36, city: "New Oslo" }
```

## defined?

**Signature** `defined?: (value) -> boolean`

**Behavior** Returns `false` when `value` is `null`, otherwise `true`. Use it to guard optional data before dereferencing or branching.

**Example**

```fip
defined?(null)
// -> false

defined?(123)
// -> true
```
