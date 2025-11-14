# HTTP

The HTTP module exposes impure helpers that perform network requests. Every helper returns a `Promise` so you can compose them with [async](./async.md) workflows. Response records expose `status`, `headers`, `text`, and `json` accessors for downstream processing. See [promise](./promise.md) to understand how the returned promises behave.

## http.request!

**Signature** `http.request!: (options) -> Promise<Response>`

**Behavior** Issues an HTTP request based on the `options` record. Expected fields include `url`, `method`, `headers`, and `body`. Missing fields fall back to sensible defaults (`method` defaults to `"GET"` and `headers` defaults to `{}`). The function returns a promise that fulfills with a response record or rejects if the request fails to reach the server.

**Implementation Notes** The builtin curries the `options` parameter, enabling higher-order factories such as `http.request!({ headers: common })`. Under the hood it defers to the host runtime's fetch implementation and preserves streaming semantics for `text` and `json`.

**Example**

```fip
async fetch-json: () {
  http.request!({
    url: "https://api.example.com/data",
    method: "PATCH",
    headers: { "content-type": "application/json" },
    body: "{\"active\": true}"
  })
  (response) { response.json.updated }
}
// -> <promise pending>
```

## http.get!

**Signature** `http.get!: (options, url) -> Promise<Response>`

**Behavior** Convenience wrapper around `http.request!` with `method: "GET"`. Accepts an `options` record and a `url` string. The `options` record can augment headers or query parameters. Returns a promise that fulfills with the response record.

**Implementation Notes** Implemented as a partially applied `http.request!`, so you can call `http.get!({}, url)` to receive a promise immediately or `http.get!({ headers: {...} }, url)` when additional headers are needed.

**Example**

```fip
async fetch-status: () {
  http.get!({}, "https://status.fip.dev/health")
  await
  (response) { response.status }
}
// -> <promise pending>
```

## http.post!

**Signature** `http.post!: (options, url, body) -> Promise<Response>`

**Behavior** Sends a POST request to `url` with the provided `body`. The `options` record can override headers or set a different content type. Returns a promise that fulfills with the response. If the network layer returns a non-2xx status, the promise still fulfills; inspect `response.status` to branch on failures.

**Implementation Notes** Sugar for calling `http.request!` with `method: "POST"` and the supplied body. Because arguments are curried, you may partially apply the options and reuse the resulting function.

**Example**

```fip
async create-user: (name) {
  http.post!(
    { headers: { "content-type": "application/json" } },
    "https://api.example.com/users",
    "{\"name\": \"" + name + "\"}"
  )
  await
  (response) { response.json.id }
}
// -> <function>

pending: create-user("Filip")
// -> <promise pending>
```

## http.put!

**Signature** `http.put!: (options, url, body) -> Promise<Response>`

**Behavior** Sends a PUT request to `url` with the provided `body`. The `options` record can override headers or set a different content type. Returns a promise that fulfills with the response. If the network layer returns a non-2xx status, the promise still fulfills; inspect `response.status` to branch on failures.

**Implementation Notes** Sugar for calling `http.request!` with `method: "PUT"` and the supplied body. Because arguments are curried, you may partially apply the options and reuse the resulting function.

**Example**

```fip
async update-user: (id, name) {
  http.put!(
    { headers: { "content-type": "application/json" } },
    "https://api.example.com/users/" + id,
    "{\"name\": \"" + name + "\"}"
  )
  await
  (response) { response.json.updated }
}
// -> <function>

result: await update-user("123", "Filip")
// -> true
```

## http.delete!

**Signature** `http.delete!: (options, url) -> Promise<Response>`

**Behavior** Sends a DELETE request to `url`. The `options` record can augment headers. Returns a promise that fulfills with the response record.

**Implementation Notes** Convenience wrapper around `http.request!` with `method: "DELETE"`. You can call `http.delete!({}, url)` to receive a promise immediately or `http.delete!({ headers: {...} }, url)` when additional headers are needed.

**Example**

```fip
async remove-user: (id) {
  http.delete!({}, "https://api.example.com/users/" + id)
  await
  (response) { response.status }
}
// -> <function>

status: await remove-user("123")
// -> 204
```
