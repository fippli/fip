# Errors

Runtime errors include a descriptive message that explains what went wrong and how to correct it. Errors do not recover automatically; they terminate the current evaluation unless tooling catches them.

## Doesn't make sense

**Signature** `Doesn't make sense: <explanation>`

**Behavior** Raised when an operator is applied to operands of incompatible types. Arithmetic operators (`+`, `-`, `*`, `/`) only accept numbers, and logical operators (`&`, `|`) only accept booleans. The error message identifies the operator and operand types.

**Example**

```fip
1 + "some string"
// -> Doesn't make sense: cannot add Number and String

true - false
// -> Doesn't make sense: cannot subtract Boolean values
```

## Suffix error

**Signature** `Suffix error: <explanation>`

**Behavior** Raised when a function definition misuses the impure `!` suffix or boolean `?` suffix. Pure bodies marked `!` and non-boolean bodies marked `?` both trigger this error. The message names the offending function.

**Example**

```fip
pure!: (x) { x + 1 }
// -> Suffix error: function marked ! but body has no impure calls

is-sum?: (x, y) { x + y }
// -> Suffix error: function marked ? but body does not return a Boolean
```

## Mutation error

**Signature** `Mutation error: trying to mutate binding <name>`

**Behavior** Raised when attempting to redefine a binding that already exists in the current scope. FIP bindings are immutable, so once a name is bound to a value, it cannot be reassigned. Use a new identifier to represent a different value.

**Example**

```fip
x: 1
x: 2
// -> Mutation error: trying to mutate binding x

count: 3
count: count + 1
// -> Mutation error: trying to mutate binding count
```

## Runtime error

**Signature** `Runtime error: <explanation>`

**Behavior** Raised for general runtime errors that do not fall into the specific categories above. This includes errors from builtin functions, undefined identifiers, type mismatches in function calls, and other runtime failures. When location information is available, the error message includes the filename and line number where the error occurred.

**Example**

```fip
undefined-value
// -> Runtime error: Undefined identifier 'undefined-value'
//    File: example.fip line 1

invalid-call: (x) { x }
invalid-call(1, 2, 3)
// -> Runtime error: Function 'invalid-call' expects 1 argument, got 3
//    File: example.fip line 2
```
