# Promise

Promises encapsulate values that may arrive later. They bridge impure effects with pure composition by representing eventual results. Read [async](./async.md) for the control-flow constructs that produce promises and [http](./http.md) for impure helpers that return them.

## Promise value

**Signature** `Promise<value>`

**Behavior** Represents a computation that is either `pending`, `fulfilled`, or `rejected`. Promises are immutable references: fulfillment fixes the value forever, and rejection records the final error. Inspecting a promise never blocks; use `await` or combinators to observe its result.

**Implementation Notes** Promises wrap a curried builtin that schedules continuations on the interpreter's event loop. Equality compares identity rather than settled values.

**Example**

```fip
async fetch-count: () {
  42
}
// -> <promise pending>
```

## Promise.resolve

**Signature** `Promise.resolve: (value) -> Promise<value>`

**Behavior** Returns a fulfilled promise containing `value`. If `value` is already a promise, the result adopts its state. Because the function is curried, you can partially apply it when building higher-order utilities.

**Implementation Notes** Implemented as a pure builtin that skips the scheduler when possible by adopting fulfilled values immediately.

**Example**

```fip
always-hello: Promise.resolve("hello")
// -> <promise fulfilled>
```

## Promise.reject

**Signature** `Promise.reject: (error) -> Promise<never>`

**Behavior** Returns a rejected promise that propagates `error` to downstream consumers. It never transitions to fulfillment.

**Implementation Notes** Stores the provided error in the promise metadata so repeated awaits throw the identical value.

**Example**

```fip
broken: Promise.reject("missing token")
// -> <promise rejected>
```

## Promise.then

**Signature** `Promise.then: (on-fulfilled, on-rejected, promise) -> Promise<result>`

**Behavior** Registers handlers that run when `promise` settles. `on-fulfilled` receives the fulfilled value, and `on-rejected` receives the error. Both handlers may return plain values or new promises; the returned value becomes the settlement of the resulting promise. To ignore rejections, pass `identity` from [core/identity](./core/identity.md) or `_` to reuse the existing error.

**Implementation Notes** The interpreter curries the three parameters, so you can supply them one at a time (for example, `Promise.then(handler)` yields a function awaiting the rejection handler and the promise).

**Example**

```fip
doubled: Promise.then(
  (value) { value * 2 },
  (error) { throw error },
  Promise.resolve(21)
)
// -> <promise fulfilled>
```

## Promise.all

**Signature** `Promise.all: (promises) -> Promise<Array<value>>`

**Behavior** Takes an array of promises (or values) and returns a promise that fulfills with an array of fulfilled values in order. If any input rejects, the returned promise rejects with the first error and cancels the remaining subscriptions.

**Implementation Notes** Uses a reference-counted latch so partial application over the input array still keeps the builtin pure until execution.

**Example**

```fip
load-both: Promise.all([async () { 1 }, async () { 2 }])
// -> <promise pending>
```

Note: Anonymous async functions use `async` before the lambda: `async () { ... }`
