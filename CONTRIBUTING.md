# Contributing

Thanks for considering a contribution. pristine is a local, read-only advisory for `docs:` changes; keep changes focused and avoid implying that its narrow path heuristic establishes author intent.

## Requirements

- Rust stable toolchain (including `cargo` and `rustfmt`)
- Git

## Checks

From the repository root, run:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Please include relevant test coverage for behavior changes and keep formatting clean.
