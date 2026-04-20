# Certus

Certus is a config-driven API gateway written in Rust. It sits in front of your
services and handles request routing, auth, caching, rate limiting,
load-balancing, and observability in one runtime. It also ships with an
embedded dashboard and internal APIs for operations.

## Highlights

- Per-route upstream proxying with HTTP/1 and HTTP/2 support
- JWT/none auth modes with route-level enforcement
- Token-bucket rate limiting (in-memory or Redis)
- Dynamic cache + static route pre-warming
- Request/cache metrics, logs, and schema capture persisted to SQLite
- Optional websocket streams for live logs and metrics

## Quick Start

### Requirements

- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/)

### 1) Build dashboard assets

`gateway/build.rs` expects a built dashboard in `dashboard/out`.

```bash
bun run --cwd dashboard build
```

### 2) Run the gateway

```bash
cargo run -p gateway -- -c examples/example.certus.config.yaml --ws logs,metrics
```

If `-c/--config` is not provided, Certus reads `certus.config.yaml` from the
current working directory.

### 3) Open the UI/API

- Dashboard: `http://localhost:8080/_certus`
- Internal API base: `http://localhost:8080/_certus/api/v1`

## Runtime Flow

Incoming requests pass through:

1. route match
2. rate limit
3. cache policy / lookup
4. auth check (if enabled for the route)
5. upstream selection and forwarding
6. metrics/log/schema recording

Config file updates are watched and hot-reloaded while the gateway is running.

## Documentation

- Configuration reference: [`docs/config.md`](docs/config.md)
- CLI reference: [`docs/cli.md`](docs/cli.md)
- Example configs: [`examples/`](examples)

## Development

Run the local dev stack:

```bash
bun dev
```

This starts the gateway with `examples/test.certus.config.yaml`, helper
upstream test servers, and the dashboard process.

## Repository Layout

- `gateway/` - Rust gateway runtime and `certus` binary
- `dashboard/` - dashboard source bundled into the gateway
- `examples/` - tracked config examples
- `test/` - helper upstream servers and benchmark/seed scripts
- `docs/` - project docs

## Release Process

Pushes to `main` build and upload workflow artifacts. Tags on `main` publish
GitHub Releases.

1. Update version in `gateway/Cargo.toml`
2. Commit and push to `main`
3. Create matching tag (for example `v1.0.0-alpha.1`)
4. Push the tag
