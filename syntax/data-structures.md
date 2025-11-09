# Data Structures

## Objects

Immutable key-value maps with string keys.

```
my-object: {
  name: "Filip",
  age: 35
}
```

### Destructuring

Access properties with dot-notation. Missing keys return `null`.

```
my-object: {
  name: "Filip",
  age: 35
}

log!(my-object.name) // "Filip"
```

```
my-object: {
  name: "Filip",
}

log!(my-object.age) // null
```

You can also destructure directly into bindings using matching object patterns. The pattern mirrors the desired keys.

```
{name}: { name: "Mefiboset" }
log!(name) // "Mefiboset"
```

Nested destructuring lets you dig into fields in one step. Unmatched keys produce `null`.

```
{person: { name, age }}: {
  person: {
    name: "Agnes",
    age: 30
  }
}
// name = "Agnes", age = 30

{person: { nickname }}: {
  person: {
    name: "Agnes"
  }
}
// nickname = null (missing key)
```

## Lists

Immutable ordered collections written with square brackets.

```
my-list: [1, 2, 5, 6]
names: ["Tore", "Knut", "Agnes", "Mefiboset"]
```

Use zero-based indexing via helper functions (e.g. `first`, `rest`) or iterate with `map`, `reduce`, and `filter`.

### Destructuring

List patterns bind each position to a new name. Extra pattern slots become `null` when the input list is shorter.

```
[first, second]: [10, 20, 30]
// first = 10, second = 20

[first, second, third]: [5]
// first = 5, second = null, third = null
```

Patterns can nest, so you can destructure lists of objects or vice versa.

```
[{ name }, { name: other-name }]: [
  { name: "Tore" },
  { name: "Knut" }
]
// name = "Tore", other-name = "Knut"

[first, { city }]: [
  [1, 2, 3],
  { city: "Oslo" }
]
// first = [1, 2, 3], city = "Oslo"
```

Destructuring is currently available for top-level assignments. Function parameters must still be plain identifiers.

```

```
