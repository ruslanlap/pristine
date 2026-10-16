# pristine

A `docs:` pull request can quietly include application code, configuration, or other changes outside documentation. Reviewing every changed path by hand is easy to forget. `pristine` gives you a local, read-only advisory when that happens.

## What it checks

Run `pristine check --base <ref> --title <title>`. When the title starts with `docs:`, pristine compares the base commit with `HEAD` and lists changed paths that do not look like documentation. It exits with status 1 when it finds any; a clean result or a title outside the rule exits 0. Usage and operational errors exit 2.

The heuristic treats paths under `docs/` and `documentation/`, files ending in `.md`, `.mdx`, `.rst`, or `.adoc`, and root-level README, CHANGELOG, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, or LICENSE filename families as documentation. It is deliberately narrow: it does not infer author intent, and a mixed-area pull request may be entirely valid. Treat findings as an advisory, not a judgment or a policy gate.

pristine inspects local Git data only. It does not use a GitHub token, network connection, or API.

## Install and use

Build from source with Rust stable and Git installed:

```sh
cargo install --path .
```

Or build and test the checkout:

```sh
cargo build
cargo test --all-targets
```

For example, from a repository with `main` as the base:

```sh
pristine check --base main --title "docs: clarify setup"
```

If that change also modifies `src/lib.rs`, pristine prints the exact path and exits 1:

```text
docs: titles should change documentation paths only; non-documentation paths found:
  src/lib.rs
```

The check is read-only; it does not change files, commits, or pull requests.

## Contributing and license

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and verification commands. This project is licensed under the [MIT License](LICENSE). Security reporting guidance is in [SECURITY.md](SECURITY.md).
