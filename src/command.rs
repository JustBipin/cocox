use crate::cli::Cli;
use crate::git_helpers::{get_commit_message_from_hash, get_commit_messages_from_hash_range};
use crate::linter::{LintOutcome, lint_commit_message};
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use anyhow::{Context, Result};

fn read_file(file: &String) -> Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))
}

fn handle_commit_message(msg: &str) {
    match lint_commit_message(msg) {
        LintOutcome::Empty => {
            std::process::exit(1);
        }
        LintOutcome::Ignored => (),
        LintOutcome::Valid => {
            println!("{}\n", VALIDATION_SUCCESSFUL);
        }
        LintOutcome::Invalid => {
            eprintln!("{}\n", VALIDATION_FAILED);
            std::process::exit(1);
        }
    }
}

fn handle_multiple_commit_messages(messages: &[String]) {
    let mut has_failure = false;

    for msg in messages {
        match lint_commit_message(msg) {
            LintOutcome::Empty => {
                std::process::exit(1);
            }
            LintOutcome::Ignored => (),
            LintOutcome::Valid => (),
            LintOutcome::Invalid => {
                has_failure = true;
            }
        }
    }

    if has_failure {
        eprintln!("{}", VALIDATION_FAILED);
        std::process::exit(1);
    }

    println!("{}", VALIDATION_SUCCESSFUL);
}

pub fn run(args: Cli) -> Result<()> {
    if let Some(msg) = &args.message {
        // direct message
        handle_commit_message(msg);
    } else if let Some(file) = &args.file {
        // commit msg file
        let msg = read_file(file)?;
        handle_commit_message(&msg);
    } else if let Some(hash) = &args.hash {
        // commit hash
        let msg = get_commit_message_from_hash(hash)?;
        handle_commit_message(&msg);
    } else if let Some(from_hash) = &args.from_hash {
        // commit hash range
        let to_hash = &args.to_hash;
        let messages = get_commit_messages_from_hash_range(from_hash, to_hash)?;
        handle_multiple_commit_messages(&messages);
    } else {
        unreachable!("invalid option is handled by clap");
    }

    Ok(())
}
