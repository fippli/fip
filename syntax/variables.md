# Variables

Fip does not support mutable variables. When you bind a name to a value, that binding is fixed for the lifetime of the scope.

## Bindings

Use `name: value` to introduce a binding. Re-binding the same name in the same scope is a compile-time error.

```
count: 3
count: 4 // ❌ cannot reassign
```

If you need a new value, create a new binding instead.

```
count: 3
next-count: count + 1 // ✅ new name
```

## Symbols

Symbol names must use kebab case: lowercase words separated with hyphens.

```
<name1>-<name2>-<name3>
```
