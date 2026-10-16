# Implementation Plan: Docs Scope Check

**Branch**: `001-docs-scope-check` | **Date**: 2026-10-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-docs-scope-check/spec.md`

## Summary

Build a local, read-only Rust CLI that compares the current branch with a user-provided base revision. For a PR title beginning exactly with `docs:`, report changed paths outside a small, documented documentation-path convention; for other titles, do not infer scope. The checker uses the installed Git executable, needs no network or GitHub credentials, and distinguishes findings from operational errors.

## Technical Context

**Language/Version**: Rust stable (MSRV not pinned for MVP)

**Primary Dependencies**: Rust standard library; Git executable available on PATH

**Storage**: N/A

**Testing**: `cargo test`; integration tests create temporary Git repositories using Git CLI

**Target Platform**: Linux, macOS, Windows with Git installed

**Project Type**: Single binary CLI

**Performance Goals**: No hard timing target in the MVP; avoid work beyond one Git diff and ordinary path classification.

**Constraints**: No network, credentials, worktree/index/ref writes, or third-party Rust crates in MVP; filenames with spaces and Unicode supported; invalid UTF-8 Git paths must fail clearly rather than be corrupted

**Scale/Scope**: One title-prefix rule, one command, human-readable output; no GitHub integration, config, JSON format, AI, or other title types

## Constitution Check

- Evidence Before Judgment: PASS — only exact `docs:` title rule; output is advisory and names paths/rule.
- Local-First and Read-Only: PASS — local Git CLI only; revision validation and diff reads must not update repo state.
- Useful Defaults, Honest Limits: PASS — no unsupported-title inference; document supported documentation paths.
- Small, Verifiable CLI: PASS — stdlib-only, deterministic logic, tests for classifications and Git behavior.
- Accessible Open Source: PASS — second task supplies README, license, contribution/security basics, and CI.

## Project Structure

### Documentation (this feature)

```text
specs/001-docs-scope-check/
├── spec.md
├── plan.md
├── research.md
├── quickstart.md
├── tasks.md
└── checklists/requirements.md
```

### Source Code

```text
.gitignore
Cargo.toml
src/main.rs       # argument parsing, Git invocation, finding presentation, exit codes
tests/cli.rs      # black-box CLI tests with temporary Git repositories
README.md
LICENSE
CONTRIBUTING.md
SECURITY.md
.github/workflows/ci.yml
```

**Structure Decision**: Single binary crate; keep the small MVP in `src/main.rs`. Integration tests exercise the compiled CLI end to end; avoid a library layer until another consumer exists.

## Design Decisions

- Command syntax: `pristine check --base <ref> --title <title>`.
- Cargo package name: `pristine-scope`; installed executable name: `pristine`.
- Compare `<base>...HEAD`, using Git's NUL-delimited name-only output to preserve spaces and Unicode. Validate the base as a commit before running the diff; invoke Git with argument arrays, never shell interpolation.
- A path is documentation if it is beneath a `docs` or `documentation` directory, has a documentation extension (`.md`, `.mdx`, `.rst`, `.adoc`), or has a recognized root-level documentation filename family (`README*`, `CHANGELOG*`, `CONTRIBUTING*`, `CODE_OF_CONDUCT*`, `SECURITY*`, `LICENSE*`). Compare filenames case-insensitively for these rules.
- For `docs:` with findings, print a warning and paths; exit 1. A clean check exits 0. Invalid invocation/ref/repository/Git failure exits 2 and prints to stderr. Unsupported titles exit 0 with an explicit “rule not applied” note.
- Deleted and renamed paths: use Git's NUL-delimited name-only list; classify the path Git reports for each changed entry. Do not attempt intent inference or rename-specific analysis.
- No JSON output, config file, GitHub API, GitHub Action, network request, or extra title rules in MVP.

## Complexity Tracking

No constitution violations. Single-file implementation is intentionally small; split only when an independently useful component emerges.
