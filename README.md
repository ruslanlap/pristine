# pristine

![CI](https://github.com/ruslanlap/pristine/actions/workflows/ci.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/pristine-scope)](https://crates.io/crates/pristine-scope)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**PR Scope & Intent Guardrail for Fast-Moving Teams & AI Agents.**  
A tiny, zero-dependency, read-only Rust CLI (`< 5ms` cold start, zero network) that catches non-matching files sneaking into `docs:`, `ci:`, `test:`, or `style:` pull requests and commits.

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ $ pristine check --base main --title "docs: fix typo in README" --agent  │
│                                                                          │
│ [PRISTINE VIOLATION] Branch/PR intent is 'docs:', but non-documentation  │
│ path(s) [src/auth.rs, .github/workflows/release.yml] were modified.      │
│ Action required: Revert [src/auth.rs, .github/workflows/release.yml]     │
│ or change the PR/commit prefix to 'feat:' / 'fix:' / 'chore:'.           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Why `pristine`?

Modern engineering teams and open-source maintainers face three growing risks:

1. **Trojan PRs & AI Slop (`xz`-style supply chain & agent drift):** A PR titled `docs: fix typo in README` modifies 20 markdown files—and quietly changes one line in `.github/workflows/release.yml`, `Cargo.toml`, or `src/auth.rs`, or plants a symlink inside `docs/` pointing to sensitive code.
2. **Broken CI Fast-Tracks:** Fast-moving teams often configure auto-merge or skip heavy 40-minute E2E test suites on `docs:` pull requests. If a developer or an AI coding agent (Claude Code, Cursor, Codex, Devin) accidentally touches application code in a `docs:` PR, broken code ships straight to production untested.
3. **Corrupted Semantic Releases:** When source code sneaks into a `docs:`, `ci:`, or `test:` PR, `release-please` / `semantic-release` skips version bumps and omits the change from release notes.

`pristine` inspects local Git data only. It requires **no GitHub token, no network connection, no write permissions, and zero runtime dependencies**—making it ideal for strict SOC2 Type II environments, local pre-push hooks, and CI pipelines.

## Why not a Bash script or Danger.js?

| Feature | Custom `git diff` Bash Script | Danger.js / Node Linters | `pristine` (Rust) |
| :--- | :--- | :--- | :--- |
| **Cold Start** | ~15ms | 800ms – 3,000ms | **< 5ms** |
| **Dependencies** | Fragile (`awk`/`grep` differences) | `node_modules` (100MB+) | **0 (Single static binary)** |
| **Unicode & Spaces in Paths** | Breaks on octal-quoted paths | Works | **Native `-z` NUL-safe parsing** |
| **Renames (`src/a.rs` → `docs/a.md`)** | Often misses old source path | Requires custom logic | **Catches both sides (`--no-renames`)** |
| **Symlinks & Submodules in `docs/`** | Blind to file mode (`120000`/`160000`) | Blind to file mode | **Flags symlinks & submodules** |
| **AI Coding Agent Feedback** | Raw file list | Markdown table for humans | **Structured `--agent` & `--format github`** |

## Built-in Scope Matrix (Conventional Commits)

`pristine` works out of the box with zero configuration for the most common non-feature Conventional Commit prefixes (including scoped variants like `docs(api):` or `ci(actions):`):

| Prefix | Allowed Paths (Zero-Config Defaults) |
| :--- | :--- |
| **`docs:`** | `docs/**`, `documentation/**`, `*.md`, `*.mdx`, `*.rst`, `*.adoc`, and root `README*`, `CHANGELOG*`, `CONTRIBUTING*`, `CODE_OF_CONDUCT*`, `SECURITY*`, `LICENSE*` |
| **`ci:`** | `.github/**`, `.circleci/**`, `.buildkite/**`, `.gitlab-ci.yml`, `Jenkinsfile`, `Dockerfile*`, `docker-compose.yml`, `action.yml`, `Makefile`, `Justfile` |
| **`test:`** | `tests/**`, `test/**`, `__tests__/**`, `e2e/**`, `benches/**`, `*_test.rs`, `*_test.go`, `*.test.ts`, `*.spec.ts`, `*.snap`, etc. |
| **`style:`** | `*.css`, `*.scss`, `*.svg`, `*.png`, `.prettierrc*`, `.editorconfig`, `rustfmt.toml`, `biome.json`, `.clang-format` |

> **Security Guarantee:** Any symlink (`120000`) or Git submodule (`160000`) added or modified—even inside `docs/`—is automatically flagged as a scope violation.

## Quick Start: GitHub Actions (1-Line Setup)

Add `pristine` to `.github/workflows/pr.yml` without installing Rust. Using `--format github` (the action's default) automatically annotates offending files directly in the GitHub PR **"Files changed"** tab:

```yaml
name: PR Scope Guard
on: [pull_request]

permissions:
  contents: read

jobs:
  scope:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Required so Git can compute the merge-base
      - uses: ruslanlap/pristine@v1
```

## Use with Lefthook / Pre-commit / Husky & AI Agents

Catch scope slip-ups locally before pushing—or let AI coding agents (Claude Code, Cursor, Codex) self-correct automatically via `--agent`:

### Lefthook (`lefthook.yml`)

```yaml
pre-push:
  commands:
    pristine:
      run: pristine check --base origin/main --title "$(git log -1 --pretty=%s)" --agent
```

### Pre-commit (`.pre-commit-config.yaml`)

```yaml
repos:
  - repo: local
    hooks:
      - id: pristine-scope
        name: Check commit scope matches changed files
        entry: bash -c 'pristine check --base HEAD~1 --title "$(git log -1 --pretty=%s)" --agent'
        language: system
        pass_filenames: false
        always_run: true
```

## Install CLI

Install from [crates.io](https://crates.io/crates/pristine-scope) with Rust stable and Git installed:

```sh
cargo install pristine-scope
```

Or download a prebuilt binary from the [latest GitHub Release](https://github.com/ruslanlap/pristine/releases/latest) (Linux x86_64, macOS Intel & Apple Silicon, Windows x86_64).

Or install from a local checkout:

```sh
cargo install --path .
```

## CLI Usage & Output Formats

```sh
Usage: pristine check --base <ref> --title <title> [options]
       pristine --help | --version

Options:
  --base <ref>            Base Git commit or branch reference
  --title <title>         Pull request or commit title (e.g. "docs: update README")
  --format <fmt>          Output format: text (default), json, github, agent
  --agent                 Shorthand for --format agent (structured AI remediation prompt)
  --allow-doc-configs     Allow documentation tool configs (.markdownlint.json, .cspell.json, etc.)
  --config <path>         Path to pristine.toml configuration file
```

### Exit Codes

- **`0`** — Clean (all changed files match the title's scope, or the title prefix is outside managed rules like `feat:` / `fix:`).
- **`1`** — Scope violation found (non-matching paths or symlinks/submodules detected).
- **`2`** — Operational or usage error (invalid arguments, not a Git repository, or missing shallow-clone history with an actionable `fetch-depth: 0` hint).

### Output Format Examples

**Human-readable (`--format text`, default):**
```text
docs: titles should change documentation paths only; non-documentation paths found:
  src/lib.rs
```

**AI Coding Agent (`--agent` or `--format agent`):**
```text
[PRISTINE VIOLATION] Branch/PR intent is 'docs:', but non-documentation path(s) [src/lib.rs] were modified. Action required: Revert [src/lib.rs] or change the PR/commit prefix to 'feat:' / 'fix:' / 'chore:'.
```

**GitHub Actions Workflow Annotations (`--format github`):**
```text
::error file=src/lib.rs,title=Pristine Scope Violation::Non-documentation file 'src/lib.rs' modified in a 'docs:' PR.
```

**Machine-readable JSON (`--format json`):**
```json
{"status":"violation","scope":"docs:","title":"docs: clarify setup","findings":["src/lib.rs"]}
```

## Custom Configuration (`pristine.toml`)

While `pristine` is zero-config by default, you can optionally drop a `pristine.toml` file in your repository root (or pass `--config <path>`) to allow custom directories, file patterns, or documentation linter configs:

```toml
allow_doc_configs = true

[docs]
allow = ["handbook/**", "rfcs/**", "*.txt"]

[ci]
allow = ["scripts/deploy/**"]

[test]
allow = ["fixtures/**"]

[style]
allow = ["design-tokens/**"]
```

## Releasing

Push a `vX.Y.Z` tag matching the version in `Cargo.toml`. The Release workflow verifies and builds Linux x86_64, macOS Intel/Apple Silicon, and Windows x86_64 archives, attaches them with checksums to a GitHub Release, then publishes to crates.io. Before the first automated publish, configure crates.io Trusted Publishing for owner `ruslanlap`, repository `pristine`, workflow `release.yml` (no environment). See the [Trusted Publishing guide](https://crates.io/docs/trusted-publishing).

## Contributing and License

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and verification commands. This project is licensed under the [MIT License](LICENSE). Security reporting guidance is in [SECURITY.md](SECURITY.md).

Try the clean and finding paths locally with [`examples/smoke.sh`](examples/smoke.sh).

