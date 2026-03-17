# Certus

## Requirements

- [Bun](https://bun.sh/)
- [Rust](https://www.rust-lang.org/)

## Gateway (Rust)

To run the gateway service:

```bash
cargo run -p gateway
```

## Releases

Gateway releases are published from version tags on `main`.

Release flow:

1. Update the version in `gateway/Cargo.toml`
2. Commit the version bump on `main`
3. Create a matching tag like `v0.1.0-alpha.1`
4. Push the tag

The release workflow builds target binaries and uploads a versioned source zip.

## Frontend (Next.js)

To run the frontend dashboard:

```bash
cd frontend
bun dev
```
