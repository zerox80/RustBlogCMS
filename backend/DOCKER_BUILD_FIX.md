# Backend Docker build

The backend requires Rust **1.88** or newer. This matches
`rust-version = "1.88"` in `Cargo.toml` and the builder image in the backend
Dockerfile.

The requirement is intentional: the committed dependency graph uses packages
whose supported compiler range starts at Rust 1.88. Do not downgrade the image
to an older compiler; it produces a build that Cargo cannot support.

## Reproducible build

The Dockerfile copies the committed `Cargo.lock` and uses `cargo build
--locked` for both cached dependencies and the final release binary. A missing
or stale lockfile therefore fails the build rather than silently changing the
dependency graph.

```bash
docker build -t minos-backend -f backend/Dockerfile backend/
```

To validate the same graph outside Docker:

```bash
cd backend
cargo +1.88.0 test --locked
cargo +1.88.0 clippy --all-targets --locked -- -D warnings
cargo audit
```

## Troubleshooting

- `Cargo.lock needs to be updated`: run `cargo update` deliberately, review
  the lockfile diff, run the checks above, then commit the lockfile.
- `rustc is not supported`: use Rust 1.88 or newer. Check with
  `rustc --version` or `docker run --rm rust:1.88-bookworm rustc --version`.
- Docker cache looks stale: rebuild with `docker build --no-cache ...`; do not
  delete the lockfile to force dependency changes.
