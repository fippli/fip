# Data Types

FIP programs operate on a small set of core value kinds. Each type enforces immutable semantics and interoperates with language helpers such as comparison operators and destructuring. This page summarizes the built-in types and shows how they behave.

## Strings

**Signature** `"text"`

**Behavior** Strings are immutable UTF-8 sequences wrapped in double quotes. Interpolation placeholders (`<binding>`) embed previously declared values during evaluation.

**Example**

```fip
name: "Filip"
// -> "Filip"

message: "My name is <name>."
// -> "My name is Filip."
```

## Numbers

**Signature** `<integer>`

**Behavior** Numbers are 64-bit signed integers. Arithmetic operators (`+`, `-`, `*`, `/`) require numeric operands and return numbers; out-of-range results raise runtime errors.

**Example**

```fip
age: 35
// -> 35

next-year: age + 1
// -> 36
```

## Boolean

**Signature** `true | false`

**Behavior** Booleans represent logical truth values. They are the only values accepted by logical operators (`&`, `|`) and by functions annotated with the `?` suffix. Equality compares by value.

**Example**

```fip
flag: true
// -> true

same: flag = true
// -> true
```

## Null

**Signature** `null`

**Behavior** `null` represents the absence of a value. Property lookups on missing keys yield `null`, and chaining continues to return `null` without raising errors. Use `defined?` to distinguish between present and missing values.

**Example**

```fip
person: { name: "Filip" }

city: person.city
// -> null

foo: null

foo.bar.baz
// -> null
```
