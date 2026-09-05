# AGENTS.md — cocox (Conventional Commit Lint for Rust)

> **Python parity is the source of truth.** The Python implementation at
> [opensource-nepal/commitlint](https://github.com/opensource-nepal/commitlint)
> is the behavioral reference for cocox. When behavior is ambiguous, inspect the
> Python source and its tests before making assumptions. Rust should be
> idiomatic, but **behavioral compatibility takes priority** over reproducing
> Python's implementation details.

## Goal

Port [commitlint](https://github.com/opensource-nepal/commitlint) (Python) to idiomatic Rust as **cocox**.
The tool lints commit messages against the [Conventional Commits](https://www.conventionalcommits.org/) standard. It is used as a CLI, GitHub Action, and pre-commit hook.

## Agent Instructions

- Preserve feature parity with the Python `commitlint` implementation unless explicitly instructed otherwise.
- Prefer small, focused changes over broad refactors.
- Do not introduce new dependencies without explicit approval.
- Do not change CLI behavior, output text, exit codes, or regex semantics without updating the corresponding parity tests.
- Do not modify completed feature-parity items unless necessary for the requested change.
- Before considering a task complete, run:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Add or update tests for every behavior change.
- Keep public/user-facing behavior compatible with the Python implementation.

## Architecture

```
src/
  main.rs          — Entry point: parse CLI, call command::run
  cli.rs           — clap CLI definition (Cli struct)
  command.rs       — Orchestration: read input, lint, display output
  config.rs        — Process-global config (LazyLock<Mutex<Config>>)
  console.rs       — Colored terminal output (success/error/verbose)
  constants.rs     — COMMIT_TYPES, IGNORE_COMMIT_PATTERNS, COMMIT_HEADER_MAX_LENGTH
  messages.rs      — All user-facing error/success strings
  linter.rs        — LintOutcome enum, LintResult, lint_commit_message()
  validators.rs    — Regex validators: simple, detailed, header-length
  utils.rs         — is_ignored, is_empty, remove_comments, remove_diff
  git_helpers.rs   — Shell out to `git show` / `git log` for hash/range queries
tests/
  cli.rs           — Integration tests (assert_cmd + predicates)
  git_helpers.rs   — Git helper unit/integration tests
  common/          — TestRepo helper (tempdir + git init)
```

### Config

`config.rs` contains the process-global configuration using `LazyLock<Mutex<Config>>`.

`ConfigGuard` exists primarily to isolate tests that temporarily modify
global configuration.

**Do not replace this with a different configuration architecture unless
explicitly requested.**

## Compatibility Requirements

Unless explicitly requested otherwise, preserve:

- CLI arguments and aliases
- Exit codes (0 success, 1 lint failure, 2 clap error)
- Error messages (exact text)
- Success/error output format
- Quiet/verbose behavior
- Commit parsing rules and regex semantics
- Ignore patterns (merge, revert, bump, initial commit, etc.)
- Header-length semantics
- Git hash/range behavior (inclusive, orphan handling)
- Comment/diff stripping behavior

## Dependencies

Do not add dependencies unless they are necessary and explicitly approved.
Prefer the Rust standard library when practical.

Current runtime dependencies:
- `clap` (derive) — CLI argument parsing
- `regex` — Commit message pattern matching
- `anyhow` — Error handling

Current development dependencies:
- `assert_cmd` — CLI integration testing
- `predicates` — Output assertion matchers
- `serial_test` — Serial test execution for global state
- `tempfile` — Temporary files/dirs for tests

## Rust Style

- Follow idiomatic Rust 2024 conventions.
- Prefer borrowing over unnecessary cloning.
- Avoid `unwrap()`/`expect()` in production code unless the invariant is genuinely guaranteed.
- Use `Result`/`Option` idiomatically.
- Keep functions focused and reasonably small.
- Avoid unnecessary abstractions.
- Prefer explicit types and straightforward control flow over clever code.

## Testing Constraints

- **`TestRepo` changes the process working directory.** Any test that uses `TestRepo` must use `#[serial]` from `serial_test`.
- **Tests involving global `Config` state** must also preserve test isolation. Use `ConfigGuard` to save/restore config.
- **Do not remove `#[serial]`** merely to make tests run concurrently.

## Feature Parity Checklist (Python → Rust)

### ✅ All Complete

- [x] CLI: positional message, `--file`, `--hash`, `--from-hash`/`--to-hash`
- [x] CLI: `--skip-detail`, `--hide-input`, `-q`/`--quiet`, `-v`/`--verbose`, `-V`/`--version`
- [x] CLI: `--max-header-length <N>` (positive integer, rejects 0)
- [x] CLI: mutual exclusion via clap ArgGroup
- [x] Config: OutputConfig (quiet/verbose), skip_detail, hide_input, strip_comments, max_header_length
- [x] Console: green/red colored output, respects quiet/verbose
- [x] Constants: 12 commit types, 9 ignore patterns, header max length (72)
- [x] Messages: all error strings matching Python (dynamic header_length_error)
- [x] Linter: LintOutcome (Valid/Invalid/Ignored/Empty), LintResult with errors
- [x] Validators: simple regex pattern, detailed pattern with per-field validation
- [x] Validators: header length check (dynamic max, not hardcoded)
- [x] Utils: is_ignored (RegexSet), is_empty, remove_diff, remove_comments
- [x] Git helpers: get_commit_message_from_hash, get_commit_messages_from_hash_range, is_orphan
- [x] Tests: 75+ integration tests covering all CLI paths, output flags, hash ranges, max-header-length
- [x] Tests: 43 unit tests for validators, utils, linter

## Validation

Run these before submitting changes:

```bash
# Formatting check
cargo fmt --check

# Tests
cargo test

# Lint (strict)
cargo clippy --all-targets --all-features -- -D warnings
```

For a quick local development cycle:

```bash
cargo fmt
cargo test
```

Run the relevant integration test suite when iterating:

```bash
cargo test --test cli
cargo test --test git_helpers
```
