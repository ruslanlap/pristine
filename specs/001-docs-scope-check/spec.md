# Feature Specification: Docs Scope Check

**Feature Branch**: `001-docs-scope-check`

**Created**: 2026-10-16

**Status**: Approved for implementation

**Input**: User selected a local advisory that warns when a `docs:` pull-request title includes changed files outside documentation; no GitHub token required.

## User Scenarios & Testing

### User Story 1 - Catch non-documentation changes in a docs PR (Priority: P1)

A contributor checks a local branch before opening a documentation-only pull request. They provide the intended PR title and base branch. When the title begins with `docs:` but the branch also changes implementation or other non-documentation files, the tool identifies those paths and explains the rule that matched.

**Why this priority**: This is the smallest concrete version of the PR-scope mismatch pain, with no remote account setup and a result that can be inspected immediately.

**Independent Test**: In a temporary Git repository, create a branch changing a documentation file and a source file, run the check with a `docs:` title and base ref, and verify the source path is reported as a warning.

**Acceptance Scenarios**:

1. **Given** a valid base ref and a `docs:` title with only documentation changes, **When** the user checks the branch, **Then** the tool reports no mismatch.
2. **Given** a valid base ref and a `docs:` title with at least one changed non-documentation file, **When** the user checks the branch, **Then** the tool reports each such path and identifies the docs-title rule.
3. **Given** a title that does not begin with the supported `docs:` prefix, **When** the user checks the branch, **Then** the tool does not apply the docs-only rule or claim the change is unrelated.

### User Story 2 - Diagnose invalid or unavailable Git inputs (Priority: P2)

A contributor supplies a missing title, invalid base ref, or runs outside a Git repository. The tool explains the problem without changing repository state.

**Why this priority**: Errors must not be confused with a clean check; safe diagnosis protects trust in the result.

**Independent Test**: Run the command with an invalid base ref and outside a repository; verify a clear error and non-success exit status for each.

**Acceptance Scenarios**:

1. **Given** the command runs outside a Git repository, **When** it tries to inspect changes, **Then** it exits with an actionable error and does not modify files or refs.
2. **Given** the supplied base ref cannot be resolved, **When** it tries to inspect changes, **Then** it reports the invalid ref rather than reporting a clean result.

## Edge Cases

- Renamed and deleted paths are evaluated from the path list Git returns for the comparison; the checker does not infer rename intent.
- Paths with spaces or non-ASCII characters are reported intact.
- An empty diff is a clean result.
- A path under a documentation directory or a recognized documentation filename/extension is treated as documentation; source and configuration paths are not.
- A `docs:` prefix is case-sensitive and must end at the conventional colon boundary; titles such as `docs-update:` do not match.

## Requirements

### Functional Requirements

- **FR-001**: Users MUST be able to check changed files between a supplied base ref and the current branch using a pull-request title.
- **FR-002**: The checker MUST apply its built-in rule only when the title begins with the exact `docs:` prefix.
- **FR-003**: The checker MUST report every changed non-documentation path for a matching `docs:` title and explain that the title indicates a documentation-only change.
- **FR-004**: The checker MUST report no mismatch when a matching title has no changed non-documentation paths.
- **FR-005**: Unsupported title prefixes MUST NOT produce inferred scope judgments.
- **FR-006**: Operational failures, including missing Git context and invalid base refs, MUST be distinguishable from a clean check and MUST identify corrective action where possible.
- **FR-007**: The check MUST be local and read-only; it MUST NOT require GitHub credentials, network access, or alter repository state.
- **FR-008**: The tool MUST support file paths containing spaces and non-ASCII characters.
- **FR-009**: A check with no mismatch MUST exit with status 0; an advisory mismatch MUST exit with status 1; invalid invocation or operational failure MUST exit with status 2 and write an actionable error to stderr.

### Key Entities

- **Check Request**: The supplied pull-request title and base revision used to inspect the current branch.
- **Changed Path**: A path reported by Git as added, modified, deleted, copied, or renamed in the comparison.
- **Mismatch Finding**: A changed non-documentation path that violates the built-in docs-title expectation; includes the path and human-readable rule explanation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: In a fixture repository, a docs-only change under a `docs:` title is reported clean, and a mixed docs/source change reports the source path correctly in 100% of automated acceptance tests.
- **SC-002**: Unsupported titles never generate scope findings in automated acceptance tests.
- **SC-003**: Invalid repository/ref inputs never produce a clean result and provide an actionable error in automated acceptance tests.
- **SC-004**: Users can run the check without creating a GitHub token, configuring an API, or granting write access.

## Assumptions

- **Assumption about scope boundaries**: The built-in documentation convention covers paths beneath `docs/` or `documentation/`, files with `.md`, `.mdx`, `.rst`, or `.adoc` extensions, and root-level README, changelog, contributing, code-of-conduct, security, or license filename families; unsupported title types are not judged.
- **Assumption about data/environment**: The user has a local Git repository and supplies the pull-request title and base revision.
- **Dependency on existing system/service**: Git must be installed and available locally; no remote service is required.
