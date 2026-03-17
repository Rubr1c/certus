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

Pushes to `main` build and upload workflow artifacts.

Version tags on `main` also publish a GitHub Release.

Release flow:

1. Update the version in `gateway/Cargo.toml`
2. Commit the version bump on `main`
3. Push the commit to `main` if you have not already
4. Create a matching tag like `v0.1.0-alpha.1`
5. Push the tag

The release workflow builds target binaries and uploads a versioned source zip.

## Frontend (Next.js)

To run the frontend dashboard:

```bash
cd frontend
bun dev
```
