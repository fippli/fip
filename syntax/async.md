# Async

Async functions return promises and enable sequential composition of asynchronous operations. Use `async` to mark functions that return promises, and `await` to wait for promise settlement. See [promise](./promise.md) for promise combinators and [http](./http.md) for network helpers that return promises.

## Async functions

**Signature** `async <name>: (<parameters>) { <body> }` or `async <name>!: (<parameters>) { <body> }`

**Behavior** Declares a function that returns a promise. The body executes asynchronously, and any promise-returning expressions can be chained using pipeline syntax. Use `!` when the function performs side effects (same rules as regular functions). Calling the function without `await` returns a pending promise; use `await` to suspend execution until the promise settles and extract the value.

**Example**

```fip
async fetch-message!: () {
  "https://status.fip.dev/health"
  http.get!
  (response) { response.json.message }
}
// -> <function>

pending: fetch-message()
// -> <promise pending>

result: await fetch-message()
// -> message
```

## Await

**Signature** `await <expression> -> value`

**Behavior** Suspends execution until the promise returned by `expression` settles. If the promise fulfills, `await` returns the fulfilled value. If it rejects, `await` throws the rejection error. `await` can only be used inside `async` functions or at the top level of a program.

**Example**

```fip
async load-data: () {
  http.get!("https://api.example.com/data")
  (response) { response.json.items }
}
// -> <function>

items: await load-data()
// -> [item1, item2, ...]
```
