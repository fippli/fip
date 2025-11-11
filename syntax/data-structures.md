# Data Structures

FIP represents structured data with immutable objects and arrays. These collections support pattern matching through destructuring and can be combined using spread syntax. This page explains how to declare, access, and decompose both forms.

## Objects

**Signature** `{ key: value, ... }`

**Behavior** Objects are immutable maps with string keys. Reassigning a field creates a new object; existing bindings remain untouched. Accessing a missing key returns `null`, which lets chained lookups short-circuit safely.

**Example**

```fip
person: {
  name: "Filip",
  age: 35
}

person.name
// -> "Filip"

person.address
// -> null
```

### Object destructuring

**Signature** `{ binding, nested: pattern, ... }: <object>`

**Behavior** Patterns bind values directly from an object into the current scope. Shorthand identifiers pull matching keys, while nested patterns allow deeper extraction. Missing keys bind to `null` so downstream code can guard with `defined?`.

**Example**

```fip
{name}: { name: "Mefiboset" }

name
// -> "Mefiboset"

{ profile: { city, country }}: {
  profile: {
    city: "Oslo",
    country: "Norway"
  }
}

city
// -> "Oslo"

country
// -> "Norway"
```

## Arrays

**Signature** `[value1, value2, ...]`

**Behavior** Arrays store ordered values and never mutate in place. Helpers like `map`, `filter`, and `reduce` return new arrays or aggregated results. Index-based helpers (`first`, `rest`, etc.) operate on zero-based positions.

**Example**

```fip
numbers: [1, 2, 5, 6]

numbers
// -> [1, 2, 5, 6]
```

### Array destructuring

**Signature** `[binding1, binding2, ...]: <array>`

**Behavior** Patterns match each array index to a binding. When the array is shorter than the pattern, the remaining bindings receive `null`. Nested patterns let you destructure arrays of objects or mix array and object extraction in the same statement. Destructuring is currently limited to top-level assignments; function parameters must stay as identifiers.

**Example**

```fip
[first, second]: [10, 20, 30]

first
// -> 10

second
// -> 20

[first, second, third]: [5]

first
// -> 5

second
// -> null

third
// -> null

[{ name }, { name: other-name }]: [
  { name: "Tore" },
  { name: "Knut" }
]

other-name
// -> "Knut"
```
