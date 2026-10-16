# Tasks: Docs Scope Check

**Input**: Design documents from `/specs/001-docs-scope-check/`

**Prerequisites**: `plan.md` and `spec.md`; `research.md` and `quickstart.md` provide supporting decisions.

**Tests**: Required by the specification; use black-box tests against the compiled CLI and temporary Git repositories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Independent files and no dependency on another task.
- **[Story]**: User story traced to `spec.md`.

## Phase 1: Setup

**Purpose**: Establish the smallest Rust binary crate.

- [ ] T001 Create `Cargo.toml` for package `pristine-scope` with binary name `pristine`, edition 2021, no third-party dependencies; verify root `.gitignore` excludes `/target/` and `/.worktrees/`, and create `src/main.rs` plus `tests/cli.rs`.

## Phase 2: Core CLI — User Story 1 (P1) and User Story 2 (P2)

**Goal**: Detect only the supported docs-title mismatch, report exact changed paths, and fail safely on Git/input errors.

**Independent Test**: `cargo test` runs the CLI against temporary Git repositories and covers clean docs-only changes, mixed changes, unsupported titles, invalid Git context/ref, and path names containing spaces and Unicode.

- [ ] T002 [US1] Write failing integration tests in `tests/cli.rs` for docs-only and empty-diff clean results; representative docs paths under `docs/`, Markdown and root-level README; mixed docs/source finding including path and rule explanation; and unsupported prefixes (`docs-update:` and `feat:`) returning no finding (FR-002–FR-005, SC-001–SC-002).
- [ ] T003 [US2] Add failing integration tests in `tests/cli.rs` for outside-repository and invalid-base errors, unchanged `HEAD` and worktree/index after a check, a mixed change containing a path with spaces and a non-ASCII character, and a deleted non-documentation path (FR-006–FR-009, SC-003).
- [ ] T004 [US1] Implement argument parsing in `src/main.rs` for `pristine check --base <ref> --title <title>` and `--help`; reject missing/unknown arguments with actionable errors (FR-001, FR-006, FR-009).
- [ ] T005 [US1] Implement documentation-path classification in `src/main.rs`: paths under `docs/` or `documentation/`; extensions `.md`, `.mdx`, `.rst`, `.adoc`; root-level basename families `README*`, `CHANGELOG*`, `CONTRIBUTING*`, `CODE_OF_CONDUCT*`, `SECURITY*`, `LICENSE*`, compared case-insensitively (FR-003–FR-004).
- [ ] T006 [US1, US2] Implement read-only Git inspection in `src/main.rs`: validate base as commit using `git rev-parse --verify --quiet --end-of-options <base>^{commit}`, run `git diff --name-only -z --no-renames <resolved-base>...HEAD`, parse NUL-delimited UTF-8 paths without shell interpolation, and treat Git/UTF-8 failures as operational errors (FR-001, FR-006–FR-008).
- [ ] T007 [US1, US2] Implement output and exit contract in `src/main.rs`: clean or unsupported title exits 0; matching `docs:` title with non-doc paths prints advisory rule plus every path and exits 1; operational/usage failures print to stderr and exit 2. Clearly state “rule not applied” for unsupported titles (FR-003–FR-006, FR-009).
- [ ] T008 [US1, US2] Run `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`; fix any failures in `Cargo.toml`, `src/main.rs`, or `tests/cli.rs` (SC-001–SC-003).

## Phase 3: Open-source launch basics

**Goal**: Make the tool understandable, installable from source, and continuously verified.

**Independent Test**: README commands match the CLI; CI workflow contains formatting, tests, and lint on Linux, macOS, and Windows; documentation and license files are present.

- [ ] T009 [P] Add a pain-first `README.md` describing the title/path mismatch, limitation, supported path convention, no-token/no-network behavior, build/test commands, and exact CLI example from `quickstart.md` (SC-004).
- [ ] T010 [P] Add the MIT `LICENSE` with copyright `2026 Ruslan Lapin`.
- [ ] T011 [P] Add `CONTRIBUTING.md` with local Rust/Git prerequisites and required format/test/clippy checks; add `SECURITY.md` with a private vulnerability-reporting route that does not invent a public email address (direct users to GitHub's private vulnerability reporting when enabled, otherwise open a private contact with the maintainer through GitHub).
- [ ] T012 Add `.github/workflows/ci.yml` with read-only `contents` permission and a stable Rust matrix for `ubuntu-latest`, `macos-latest`, and `windows-latest`; run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
- [ ] T013 Verify README commands against the built CLI, then rerun `cargo fmt --check`, `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, and `git diff --check`.

## Dependencies & Execution Order

- T001 must precede T002–T008 because tests and implementation require the crate.
- T002 and T003 specify the failing acceptance tests before the implementation in T004–T007.
- T004 and T005 precede T006–T007; all core tasks precede T008.
- T009–T011 are independent after the product interface is fixed; T012–T013 follow core implementation, and T013 verifies the integrated repository.
- The open-source launch task group starts only after Phase 2 passes.

## Implementation Strategy

1. Complete T001.
2. Write and run T002–T003 tests to establish expected failures.
3. Implement T004–T007; run the targeted tests after each behavior, then the full checks in T008.
4. Add T009–T012 as a separate reviewed change after core verification.
5. Run T013 and inspect the final diff before considering the MVP complete.

## Parallel Opportunities

- After T008, T009, T010, and T011 may be handled independently by separate workers; T012 can proceed independently once CLI contract is fixed. Integration/verification stays centralized in T013.
