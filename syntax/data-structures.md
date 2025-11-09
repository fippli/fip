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

## Lists

Immutable ordered collections written with square brackets.

```
my-list: [1, 2, 5, 6]
names: ["Tore", "Knut", "Agnes", "Mefiboset"]
```

Use zero-based indexing via helper functions (e.g. `first`, `rest`) or iterate with `map`, `reduce`, and `filter`.
