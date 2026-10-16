# Quickstart: Docs Scope Check

## Build and test

```sh
cargo test
cargo build --release
```

## Check a local branch

From a Git repository, give the target base and the intended pull-request title:

```sh
pristine check --base origin/main --title "docs: update installation guide"
```

A docs-only change exits successfully. If that title includes non-documentation paths, the CLI lists each path, names the rule, and exits with a finding status. Unsupported title prefixes are reported as “rule not applied”; they do not produce inferred findings. Invalid Git context or arguments are errors, not clean results.

The check is local and read-only. It requires Git on `PATH`; it does not use a GitHub token, API, or network connection.

## Initial documentation convention

Paths under `docs/` or `documentation/`, paths with `.md`, `.mdx`, `.rst`, or `.adoc` extensions, and root-level `README*`, `CHANGELOG*`, `CONTRIBUTING*`, `CODE_OF_CONDUCT*`, `SECURITY*`, or `LICENSE*` files count as documentation. This is a deliberately small default convention, not a claim about every repository's intent.
