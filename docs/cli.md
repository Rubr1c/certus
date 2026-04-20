# CLI Arguments

This project exposes the `certus` binary from `gateway`.
This doc assumes the binary is already built and available as `./certus`.

Run with:

```bash
./certus [ARGS]
```

## Available args (current)

- `-c, --config <PATH>`
  - YAML config file path.
  - Default: `certus.config.yaml`.

- `--save`
  - Persists config snapshots to SQLite during state initialization/reload.
  - Default: disabled.

- `--ws <logs|metrics>[,<logs|metrics>...]`
  - Enables websocket endpoints for realtime streams.
  - Supported values:
    - `logs` -> enables `/_certus/api/v1/ws/logs`
    - `metrics` -> enables `/_certus/api/v1/ws/metrics`
  - Multiple values can be passed comma-separated.

## Clap built-ins

- `-h, --help`: prints help.
- `-V, --version`: prints version.

## Examples

```bash
# Run with a custom config file
./certus --config examples/example.certus.config.yaml

# Enable logs websocket
./certus -c examples/example.certus.config.yaml --ws logs

# Enable both logs and metrics websockets
./certus -c examples/example.certus.config.yaml --ws logs,metrics

# Save config snapshots to SQLite
./certus -c examples/example.certus.config.yaml --save
```
