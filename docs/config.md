# Configuration Reference

This project reads a YAML gateway config file (`certus.config.yaml` by default).
The file is parsed into `Config` and can be hot-reloaded while the process is
running.

This doc assumes the binary is already built and run directly, for example:

```bash
./certus -c examples/example.certus.config.yaml
```

For complete examples, see:

- `examples/example.certus.config.yaml`
- `examples/test.certus.config.yaml`

## Top-level schema

```yaml
server:
  port: 8080
  origins: []

routes: {}

tls:
  cert_path: examples/certs/cert.pem
  key_path: examples/certs/key.pem

auth:
  method: !none
  prefix: Bearer

rate_limit:
  max_tokens: 100
  refill_rate: 1
  key: !ip
  type: !in_memory

connection:
  connect_timeout: 2000

cache:
  size: 1000
  type: !in_memory
  ttl: null
  tti: null
  max_size: 10485760
  static:
    type: !in_memory

default_server: 127.0.0.1:80
```

## `server`

- `server.port` (default: `8080`): port Certus binds to.
- `server.origins` (default: `[]`): allowed CORS origins for incoming requests.
  Empty means allow all origins.

## `routes`

`routes` is a map where each key is a route prefix and each value is route
behavior.

Example:

```yaml
routes:
  /api/users:
    endpoints: ["127.0.0.1:3101", "127.0.0.1:3102"]
    needs_auth: true
    token_weight: 1
```

Per-route options:

- `endpoints` (required): list of upstream addresses.
  - Supports `host:port`, `http://host:port`, or `https://host:port`.
  - If scheme is missing, HTTP is assumed.
- `is_static` (default: `false`): prefetches a GET response for this route at
  startup/reload and serves it from static cache.
- `needs_auth` (default: `false`): requires authentication for this route.
- `http_version` (default: `HTTP1`): upstream protocol version (`HTTP1` or
  `HTTP2`).
- `max_connections` (default: `100`): max pooled connections per upstream.
- `token_weight` (default: `0`): rate-limit token cost per request on this
  route.
- `no_cache` (default: `false`): bypasses dynamic response caching for this
  route.

Notes:

- Route matching supports both the exact route and nested paths (for example
  `/api` also matches `/api/items/123`).
- Upstream selection uses idle queue first, then power-of-two-choices (least
  loaded pick between two random endpoints).

## `tls`

- `tls` is optional. If omitted, Certus serves plain HTTP.
- `tls.cert_path`: PEM certificate path.
- `tls.key_path`: PEM private key path.

When `tls` is present and valid, the gateway serves HTTPS.

## `auth`

- `auth.prefix` (default: `Bearer`): expected authorization prefix in
  `Authorization` header.
  - Example: `Authorization: Bearer <token>`.
- `auth.method`: authentication strategy.
  - `!none`: no token verification.
  - `!jwt`: JWT verification with:
    - `secret` (required): signing secret.
    - `algorithm` (default: library default): JWT algorithm such as `HS256`.

Auth is only enforced on routes with `needs_auth: true`.

## `rate_limit`

- `rate_limit.max_tokens` (default: `100`): bucket capacity.
- `rate_limit.refill_rate` (default: `1`): tokens added per second.
- `rate_limit.key` (default: `!ip`): identity used per bucket.
  - `!ip`: bucket per client IP.
  - `!token`: bucket per auth token (falls back to IP if token missing).
  - `!header "Header-Name"`: bucket per header value (falls back to IP if
    header missing).
- `rate_limit.type` (stored as `rl_type`, default: `!in_memory`): backend.
  - `!in_memory`: local process cache.
  - `!redis` with `url`: shared Redis backend.

`token_weight` on each route is the per-request cost from the bucket.

## `connection`

- `connection.connect_timeout` (default: `2000`): upstream connect timeout value
  used by forwarding and static prefetch.

Note: this value is currently treated as seconds by the connector code, so keep
it small in practice.

## `cache`

- `cache.size` (default: `1000`): max entry capacity for dynamic cache.
- `cache.type` (stored as `cache_type`, default: `!in_memory`): dynamic cache
  backend.
  - `!in_memory`
  - `!redis` with `url`
- `cache.ttl` (default: `null`): time-to-live in seconds.
- `cache.tti` (default: `null`): time-to-idle in seconds.
- `cache.max_size` (default: `10485760`): maximum response body bytes to store.
- `cache.static.type`: static cache backend declaration in config shape.

Notes:

- Dynamic cache stores GET success responses.
- Cache bypass happens for non-GET, `no_cache` routes, private tokenized
  requests (unless explicitly `public`), or restrictive cache-control headers.
- Static cache is warmed for routes with `is_static: true`.

## `default_server`

- `default_server` (default: `127.0.0.1:80`): fallback upstream address when a
  route has no usable endpoint.

## Runtime behavior

- The config file is watched for modifications and hot-reloaded.
- `GET /_certus/api/v1/config` returns the active in-memory config snapshot.
- `PUT /_certus/api/v1/config` updates config in memory and persists it to
  SQLite.
- If started with `--save`, each reinitialization also writes the config to DB.
