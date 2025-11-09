# Data Types

## Strings

Immutable sequences of UTF-8 characters enclosed in double quotes.

```
name: "Filip"
```

### Replacements `"<x>"`

Use `<identifier>` placeholders to interpolate existing bindings into string literals.

```
name: "Filip"
sentence: "My name is <name>."
```

## Numbers

64-bit signed integers. Arithmetic operators operate strictly on numeric values.

```
age: 35
```

## null

Represents the absence of a value. Returned by the runtime when a lookup fails or a computation cannot produce a result.

```
object: { name: "Filip" }
result: object.age // null
```

`null` is chainable; property access on `null` always returns `null`.

```
foo: null
foo.bar.baz // null
```

## Boolean

`true` or `false`
