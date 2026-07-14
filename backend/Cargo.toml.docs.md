# Backend Cargo configuration

`Cargo.toml` defines the `minos-backend` application crate and two content
utilities (`export_content` and `import_content`). The project uses Rust 2021
with **Rust 1.88** as its minimum supported compiler.

## Dependency policy

- `axum`, `tokio`, `tower-http`, and `tower_governor` provide the HTTP stack
  and rate limiting.
- `sqlx` is intentionally limited to SQLite with `default-features = false`;
  MySQL and PostgreSQL drivers are not compiled or shipped.
- `jsonwebtoken = 9.3.1` is pinned and configured without PEM support because
  the application uses symmetric HMAC keys only.
- `dotenvy` loads local development environment files; `sha2`, `hmac`, and
  `subtle` support authentication and CSRF primitives.
- `home` and `base64ct` are exact pins. Re-evaluate them together with the
  MSRV before upgrading dependencies.

## Reproducible commands

```bash
cd backend
cargo +1.88.0 fmt --check
cargo +1.88.0 test --locked
cargo +1.88.0 clippy --all-targets --locked -- -D warnings
cargo audit
```

`Cargo.lock` is committed and required by the Docker build. Use `cargo update`
only as a deliberate dependency update, review the resulting lockfile diff,
then rerun the commands above.
