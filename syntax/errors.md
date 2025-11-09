# Errors

Runtime errors include a descriptive message that explains what went wrong and how
to correct it. Errors do not recover automatically; they terminate the current
evaluation unless caught by higher-level tooling.

## Doesn't make sense

Raised when an operator is applied to operands of incompatible types, such as
adding a number to a string. Arithmetic operators (`+`, `-`, `*`, `/`) only
accept numbers, while logical operators (`&`, `|`) only accept booleans.

**Message format:** `Doesn't make sense: <explanation>`

**Example**

```
1 + "some string"
// Doesn't make sense: cannot add Number and String
```

```
true - false
// Doesn't make sense: cannot subtract Boolean values
```

## Suffix error

Raised when a function definition uses the impure `!` suffix or boolean `?`
suffix incorrectly.

**Message format:** `Suffix error: <explanation>`

**Examples**

```
pure!: (x) { x + 1 }
// Suffix error: function marked ! but body has no impure calls
```

```
is-sum?: (x, y) { x + y }
// Suffix error: function marked ? but body does not return a Boolean
```
