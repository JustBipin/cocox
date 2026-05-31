mod common;

use assert_cmd::Command;
use common::TestRepo;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn cocox() -> Command {
    Command::cargo_bin("cocox").expect("cocox binary should be built")
}

fn write_temp(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    file.write_all(contents.as_bytes())
        .expect("write temp file");
    file
}

// --- positional message ----------------------------------------------------

#[test]
fn valid_message_succeeds() {
    cocox()
        .arg("feat: add new feature")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn valid_message_with_scope_succeeds() {
    cocox()
        .arg("fix(parser): handle empty input")
        .assert()
        .success();
}

#[test]
fn valid_message_with_body_succeeds() {
    cocox()
        .arg("feat: add new feature\n\nthis is the body")
        .assert()
        .success();
}

#[test]
fn invalid_message_fails() {
    cocox()
        .arg("not a conventional commit")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn unknown_type_fails() {
    cocox().arg("wip: something").assert().failure().code(1);
}

#[test]
fn empty_message_aborts() {
    cocox()
        .arg("")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

#[test]
fn whitespace_only_message_aborts() {
    cocox()
        .arg("   \n\t  ")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

// --- ignored messages ------------------------------------------------------
//
// These don't match the linter regex but should silently succeed because
// `command::handle_commit_message` short-circuits via `utils::is_ignored`.

#[test]
fn merge_commit_is_ignored() {
    cocox()
        .arg("Merge pull request #123")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn dependabot_bump_is_ignored() {
    cocox()
        .arg("Bump urllib3 from 1.26.5 to 1.26.17")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn initial_commit_is_ignored() {
    cocox().arg("Initial commit").assert().success();
}

// --- --file ---------------------------------------------------------------

#[test]
fn file_with_valid_message_succeeds() {
    let file = write_temp("feat: add new feature");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn file_with_invalid_message_fails() {
    let file = write_temp("bad commit message");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn file_with_empty_contents_aborts() {
    let file = write_temp("");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

#[test]
fn file_with_whitespace_only_content_aborts() {
    let file = write_temp("   \n\t  ");
    cocox()
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Aborting commit due to empty commit message",
        ));
}

#[test]
fn missing_file_fails() {
    cocox()
        .arg("--file")
        .arg("/nonexistent/path/commit-msg.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to read commit message file",
        ));
}

// --- --hash ---------------------------------------------------------------

#[test]
fn hash_head_valid_commit_succeeds() {
    let repo = TestRepo::new();
    repo.commit("feat: add new feature");

    cocox()
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn invalid_hash_fails() {
    cocox()
        .arg("--hash")
        .arg("0000000000000000000000000000000000000000")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to retrieve commit message for hash",
        ));
}

#[test]
fn hash_exact_valid_commit_succeeds() {
    let repo = TestRepo::new();
    let hash = repo.commit("feat: add new feature");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn hash_invalid_commit_message_fails() {
    let repo = TestRepo::new();
    let hash = repo.commit("not a conventional commit");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn hash_ignored_commit_silently_succeeds() {
    let repo = TestRepo::new();
    let hash = repo.commit("Merge pull request #123");

    cocox()
        .arg("--hash")
        .arg(&hash)
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

// ---- hash range ----------------------------------------------------------

#[test]
fn hash_range_valid_commits_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature A");
    repo.commit("fix: fix a bug");
    repo.commit("fix: fix another bug");
    repo.commit("feat: add another feature");
    let c = repo.commit("perf: improve performence");

    cocox()
        .arg("--from-hash")
        .arg(a)
        .arg("--to-hash")
        .arg(c)
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn hash_range_invalid_commit_in_middle_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    repo.commit("not a conventional commit");
    let c = repo.commit("fix: fix a bug");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&c)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn hash_range_invalid_from_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("not a conventional commit");
    let b = repo.commit("feat: add new feature");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&b)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn hash_range_invalid_to_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    let b = repo.commit("not a conventional commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&b)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn hash_range_ignored_commits_are_skipped() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature");
    repo.commit("Merge pull request #123");
    let c = repo.commit("fix: fix a bug");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&c)
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn hash_range_single_commit_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: single commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&a)
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

#[test]
fn hash_range_single_invalid_commit_fails() {
    let repo = TestRepo::new();

    let a = repo.commit("not a conventional commit");

    cocox()
        .arg("--from-hash")
        .arg(&a)
        .arg("--to-hash")
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Commit validation: failed!"));
}

#[test]
fn from_hash_only_valid_commits_succeeds() {
    let repo = TestRepo::new();

    let a = repo.commit("feat: add new feature A");
    repo.commit("fix: fix a bug");
    repo.commit("fix: fix another bug");
    repo.commit("feat: add another feature");
    repo.commit("perf: improve performence");

    cocox()
        .arg("--from-hash")
        .arg(a)
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit validation: successful!"));
}

// --- clap argument constraints --------------------------------------------

#[test]
fn no_args_fails_with_clap_error() {
    cocox()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required"));
}

#[test]
fn to_hash_only_fails() {
    cocox()
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required"));
}

#[test]
fn message_and_file_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("feat: something")
        .arg("--file")
        .arg(file.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn file_and_hash_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("--file")
        .arg(file.path())
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn file_and_from_hash_together_fail() {
    let file = write_temp("feat: a body");
    cocox()
        .arg("--file")
        .arg(file.path())
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn message_and_hash_together_fail() {
    cocox()
        .arg("feat: something")
        .arg("--hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn message_and_from_hash_together_fail() {
    cocox()
        .arg("feat: something")
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn hash_and_from_hash_together_fail() {
    cocox()
        .arg("--hash")
        .arg("HEAD")
        .arg("--from-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn to_hash_with_hash_together_fail() {
    // --to-hash requires --from-hash specifically, not just any input arg
    cocox()
        .arg("--hash")
        .arg("HEAD")
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn to_hash_with_message_fails() {
    // --to-hash requires --from-hash, passing a message doesn't satisfy it
    cocox()
        .arg("feat: something")
        .arg("--to-hash")
        .arg("HEAD")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn version_flag_succeeds() {
    cocox()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cocox"));
}

#[test]
fn help_flag_succeeds() {
    cocox()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Conventional Commitlint"));
}
