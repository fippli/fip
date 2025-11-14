# Test Server

A minimal HTTP test server for testing async features in the Fippli language.

## Usage

Start the server:

```bash
cargo run
```

The server listens on `http://127.0.0.1:3000`.

## Endpoints

### `GET /health`

Returns a simple health check response.

**Response:**

```json
{
  "method": "GET",
  "path": "/health",
  "status": 200,
  "message": "OK"
}
```

### `POST /echo`

Echoes the request body back in the response.

**Response:**

```json
{
  "method": "POST",
  "path": "/echo",
  "status": 200,
  "body": "<request body>"
}
```

### `GET /delay`

Simulates a delayed response (100ms delay).

**Response:**

```json
{
  "method": "GET",
  "path": "/delay",
  "status": 200,
  "message": "delayed response"
}
```

### `GET /error`

Returns a 500 Internal Server Error.

**Response:**

```json
{
  "error": "test error"
}
```

## Testing with Fippli

Example Fippli code to test the server:

```fip
fetch-health: async () {
  http.get!("http://127.0.0.1:3000/health")
  await
  (response) { response.json.message }
}

result: await fetch-health()
// -> "OK"
```
