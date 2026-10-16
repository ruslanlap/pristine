# pristine Constitution

## Core Principles

### I. Evidence Before Judgment
The tool reports only explainable heuristics over explicit inputs. It must never claim to know author intent or label a change unrelated as fact.

### II. Local-First and Read-Only
Core checks run locally against Git data, require no GitHub credentials or network access, and never modify the working tree or repository.

### III. Useful Defaults, Honest Limits
The first release has one narrow built-in rule: a `docs:` pull-request title paired with changed non-documentation files. Findings are advisory, identify the triggering rule and files, and avoid guessing for unsupported title types.

### IV. Small, Verifiable CLI
Use Rust's standard library and Git CLI unless a concrete requirement justifies a dependency. Every behavior affecting results has automated tests; errors are actionable and written to stderr.

### V. Accessible Open Source
Provide a concise README with a runnable demo, installation/build instructions, license, contribution guidance, and CI for supported platforms before public launch.

## Technical Constraints

- Implement as a cross-platform Rust command-line application.
- Read Git state without changing refs, index, or worktree.
- Do not add GitHub API or AI/network inference to the MVP.
- Keep warnings distinct from operational errors through documented exit behavior.

## Development Workflow

- Follow GitHub Spec Kit artifacts in `specs/` for product requirements and implementation sequencing.
- Run formatting, tests, and lint checks before considering a task complete.
- Keep commits and changes focused; do not commit credentials or machine-specific state.

## Governance

This constitution applies to all project work and overrides lower-level plans. Amendments require an explicit documented change and must not silently broaden the MVP. Simplicity does not excuse inaccurate findings, unsafe Git operations, or missing verification.

**Version**: 1.0.0 | **Ratified**: 2026-10-16 | **Last Amended**: 2026-10-16
