# AGENTS.md

Instructions for AI coding agents working on cocox. This is the single source of truth for
agent guidance. Tool-specific files, such as `CLAUDE.md`, only point here.

Read [CONTRIBUTING.md](./CONTRIBUTING.md) first for setup, the commands to run, and the
pull request process. This file covers only what an agent gets wrong by default.

Everything below was verified by running it. Keep it that way. A confident but wrong
instruction is worse than a missing one, because the next contributor trusts it.

## The parity contract

cocox is a Rust port of [commitlint](https://github.com/opensource-nepal/commitlint). The
Python tool is the reference. If the two disagree for the same input, cocox is wrong,
unless a maintainer decides otherwise in writing.

Prove parity, never assume it. Run both and compare the exit code, stdout and stderr:

```bash
pip install commitlint==2.0.0
cargo build
./target/debug/cocox "feat:" ; echo "exit=$?"
commitlint "feat:"           ; echo "exit=$?"
```

Upstream's own table of cases is at `tests/fixtures/linter.py` in its repository. It is
ported here as `tests/upstream_parity.rs`.

Two upstream behaviors are defects, not contracts. Do not copy them: upstream raises
`IndexError` on an empty or comment-only `--file`, and it accepts an arbitrarily large
`--max-header-length`.

## Porting traps

Python and Rust look alike at each of these points and behave differently. Each one has
produced a real defect, either on `main` or in a pull request against it.

1. **Count characters, not bytes.** Python `len()` on a str counts code points; Rust
   `.len()` counts UTF-8 bytes. `"feat: नमस्ते संसार"` is 18 characters and 40 bytes. Use
   `.chars().count()`. `clippy.toml` denies `str::len` for this reason.
2. **A missing regex group is falsy, not an early return.** Python `m.group("x")` is
   `None` when the group did not participate, and `if not ...` catches it. In Rust,
   `captures.name("x")?` returns from the whole function and the error is never recorded.
   Write `captures.name("x").map_or("", |m| m.as_str())` instead.
3. **Normalize newlines where you read input.** Upstream reads git output with
   `text=True` and files with `open()`, so Python converts `\r\n` and lone `\r` to `\n`
   before linting. cocox reads raw bytes. Normalize in the readers only. Upstream does not
   normalize a message passed as a command-line argument, so neither must cocox.
4. **Anchor every regex you port.** Python `re.match` anchors at position 0; the Rust
   `regex` crate searches unless you write `^`. Note also that Rust `$` does not match
   before a trailing newline the way Python `$` does.
5. **`splitlines()` is not `lines()`.** Python also breaks on `\r`, `\x0b`, `\x0c`,
   `\x1c`, `\x1d`, `\x1e`, `\x85`, U+2028 and U+2029. Rust `lines()` breaks on `\n` only.
6. **`.strip()` is not `.trim()`.** Trim at exactly the places upstream strips, and note
   that Python treats `\x1c` to `\x1f` as whitespace while Rust does not.
7. **Constant order is user-visible.** Upstream joins `COMMIT_TYPES` into the
   "Type must be one of: ..." error, so the array order becomes text a user reads. Keep
   ported constant arrays in upstream order even when nothing prints them yet.
8. **Send every user-facing line through one output path.** Upstream routes everything
   through its console layer, which returns early when quiet is set, and catches expected
   failures. An error that escapes to `main` prints an anyhow chain that no quiet flag can
   suppress.
9. **Port an argparse type function into a clap `value_parser`.** Reproduce upstream's
   error text, and add `allow_negative_numbers = true`, or clap treats `-5` as a flag and
   the parser never runs.
10. **Pass lint options as arguments.** Keep `lint_commit_message` a function of its
    arguments, as upstream does with its `AppParams` dataclass. A process global makes the
    same input return different results depending on what ran before, and makes unit tests
    race.

## Testing rules

- Assert the exact message, not just the exit code. cocox and upstream both exit 2 for
  every bad argument, so an exit code alone proves nothing.
- Prefer an exact match over a substring. `contains("")` is true for every string.
- Never delete a test to make a change pass. Port it, or say in the pull request why the
  behavior it pinned is gone.
- Add the input that broke to `tests/cli.rs` when you fix a parity bug. Do not add it to
  `tests/upstream_parity.rs`, which is a verbatim mirror of upstream's table and asserts
  its own row count, so a local row makes it fail.
- `TestRepo` changes the working directory of the whole test process, so any test using it
  needs `#[serial]`. Do not remove that attribute to make tests faster.

## Known divergences from upstream

Open defects, not decisions. Each was reproduced against commitlint 2.0.0. Fix them in
separate pull requests. This list is what has been measured, not everything that exists.

1. A leading space fails here and passes upstream, because the direct message and
   `--file` contents are never stripped.
2. A CRLF message file fails here and passes upstream.
3. `Initial commit\x0cgarbage` is linted here and ignored upstream, per trap 5.
4. A trailing `\x1c` to `\x1f` survives `trim()` here and is stripped upstream.
5. `COMMIT_TYPES` lists `bump` second; upstream lists it last, per trap 7.
6. A missing `--file` prints a multi-line anyhow chain; upstream prints one line,
   `Error: file '<path>' not found`.

## Working rules

- **Keep scratch work out of the repository.** Put probe crates, harness scripts and
  clones of upstream in a temporary directory. Never commit one. To compare against main
  without disturbing the checkout:
  `mkdir -p /tmp/cocox-main && git archive main | tar -x -C /tmp/cocox-main`
- **Do not write unverified claims into the repository.** Check every factual sentence you
  add to a document against the code or a command. Do not write that a gate, a test or a
  feature exists until you have seen it run. Prefer "measured so far" over "complete".
- **Report what you actually ran.** If tests fail, say so and show the output. If you
  skipped a step, say which. Do not describe work as done until it is done.
- **One concern per pull request.** Do not add repository policy to a feature pull
  request. Propose policy separately so it can be discussed on its own.

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
