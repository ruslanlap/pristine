# Research: Docs Scope Check

## Product shape

A generic CLI cannot know whether a changed file is “unrelated” without explicit intent or repository-specific rules. Existing GitHub Actions already enumerate changed files and apply include/exclude patterns. The defensible MVP is narrower: detect a mismatch between a conventional `docs:` PR title and changes outside documentation paths; make the result advisory and explainable.

## Adjacent tools and evidence

- [`tj-actions/changed-files`](https://github.com/marketplace/actions/changed-files) computes changed paths and supports include/exclude patterns; it does not infer fit with a PR title.
- [`check-for-changed-files`](https://github.com/marketplace/actions/check-for-changed-files) checks for required matching files; it does not report unexpected paths based on title intent.
- [`github-pr-contains-action`](https://github.com/marketplace/actions/check-pull-request-body-diff-and-files) supports configured conditions and size limits; it is GitHub Action-oriented rather than a local no-token check.
- [`action-semantic-pull-request` issue #186](https://github.com/amannn/action-semantic-pull-request/issues/186) discusses title-type/path mismatch as an adjacent request and notes mixed-area changes still need human review.

## Decisions

- Keep only one rule (`docs:` title + non-documentation path) for v1.
- Never call a path unrelated; say “outside documentation paths” and show the exact paths/rule.
- Run locally, read-only, without network or token; accept explicit title/base inputs.
- Treat this as a workflow helper, not evidence of proven market demand. Validate utility with real PRs before broadening.
- Keep the executable name `pristine`, but publish under the distinct Cargo package name `pristine-scope`: crates.io already has [`pristine`](https://crates.io/crates/pristine), a Ruby formatter. `pristine-scope` returned no result in the live `cargo search` check; availability can change before publishing.

## Risks

- Cross-cutting changes legitimately combine docs and code, so advisory wording and exact path display are mandatory.
- Extension/directory conventions vary by repository; a built-in classifier is imperfect. Keep it documented and bounded; add configuration only if users demonstrate repeat demand.
- Name availability and adoption demand are unverified; do not claim popularity or uniqueness.
