# Builtin Functions Rule

## All Builtin Functions Must Support Currying

All builtin functions in the FIP language must support currying (partial application). This means:

1. **Every builtin function must have a `params` field** in the `BuiltinFunction` struct that lists all parameter names in order.

2. **When a builtin is called with fewer arguments than required**, it must return a curried function that captures the provided arguments and waits for the remaining ones.

3. **When a curried builtin function is called**, it must combine the captured arguments with the new arguments and call the original builtin when all required arguments are provided.

## Implementation Requirements

- The `BuiltinFunction` struct must include: `params: Vec<String>`
- The `call_callable` method must check `args.len() < builtin.params.len()` to detect partial application
- Curried builtin functions must store `__curried_builtin__` and `__curried_args__` in their environment
- The function call path must handle curried builtin functions by combining arguments and calling the original builtin

## Example

```rust
self.add_builtin(BuiltinFunction {
    name: "trace!".to_string(),
    impure: true,
    params: vec!["label".to_string(), "value".to_string()], // Required!
    func: Rc::new(|interpreter, args| {
        // Implementation
    }),
});
```

## Testing

When adding a new builtin function, verify that:

- Calling it with fewer arguments returns a function
- The returned function can be called with the remaining arguments
- The final result matches calling the builtin with all arguments at once

Example test:

```fip
trace-x: trace!("x")  // Should return a function
result: trace-x("test")  // Should work and print "(trace) x: test"
```
