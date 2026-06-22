use crate::cli::Cli;
use crate::config::OutputConfig;
use crate::console;
use crate::git_helpers::{get_commit_message_from_hash, get_commit_messages_from_hash_range};
use crate::linter::{LintOptions, LintOutcome, lint_commit_message_with_options};
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use crate::utils::remove_diff_from_commit_message;
use anyhow::{Context, Result};

struct HandlerOptions {
    output: OutputConfig,
    skip_detail: bool,
    hide_input: bool,
    strip_comments: bool,
}

impl HandlerOptions {
    fn from_cli(args: &Cli) -> Self {
        Self {
            output: OutputConfig::new(args.quiet, args.verbose),
            skip_detail: args.skip_detail,
            hide_input: args.hide_input,
            strip_comments: false,
        }
    }

    fn lint_options(&self) -> LintOptions {
        LintOptions {
            skip_detail: self.skip_detail,
            strip_comments: self.strip_comments,
        }
    }
}

fn read_file(file: &str) -> Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))
        .map(|content| content.trim().to_string())
}

fn show_errors(message: &str, errors: &[String], options: &HandlerOptions) {
    let message = remove_diff_from_commit_message(message);

    if !options.hide_input {
        console::error(&format!("⧗ Input:\n{message}\n"), &options.output);
    }

    if options.skip_detail {
        console::error(VALIDATION_FAILED, &options.output);
        return;
    }

    console::error(
        &format!("✖ Found {} error(s).", errors.len()),
        &options.output,
    );
    for error in errors {
        console::error(&format!("- {error}"), &options.output);
    }
}

fn handle_commit_message(message: &str, options: &HandlerOptions) {
    console::verbose("linting commit message:", &options.output);
    console::verbose(
        &format!("----------\n{message}\n----------"),
        &options.output,
    );

    let result = lint_commit_message_with_options(message, options.lint_options());

    match result.outcome {
        LintOutcome::Empty => std::process::exit(1),
        LintOutcome::Ignored => {
            console::verbose("commit message ignored, skipping lint", &options.output);
        }
        LintOutcome::Valid => {
            console::success(VALIDATION_SUCCESSFUL, &options.output);
        }
        LintOutcome::Invalid => {
            show_errors(message, &result.errors, options);
            std::process::exit(1);
        }
    }
}

fn handle_multiple_commit_messages(messages: &[String], options: &HandlerOptions) {
    let mut has_error = false;

    for message in messages {
        let result = lint_commit_message_with_options(message, options.lint_options());

        match result.outcome {
            LintOutcome::Empty => std::process::exit(1),
            LintOutcome::Ignored => {
                console::verbose("lint success", &options.output);
            }
            LintOutcome::Valid => {
                console::verbose("lint success", &options.output);
            }
            LintOutcome::Invalid => {
                has_error = true;
                show_errors(message, &result.errors, options);
                console::error("", &options.output);
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }

    console::success(VALIDATION_SUCCESSFUL, &options.output);
}

pub fn run(args: Cli) -> Result<()> {
    let mut options = HandlerOptions::from_cli(&args);

    console::verbose("starting cocox", &options.output);

    if let Some(message) = &args.message {
        console::verbose("commit message source: direct message", &options.output);
        handle_commit_message(message.trim(), &options);
    } else if let Some(file) = &args.file {
        console::verbose("commit message source: file", &options.output);
        let abs_path = std::fs::canonicalize(file)
            .unwrap_or_else(|_| std::path::PathBuf::from(file))
            .display()
            .to_string();
        console::verbose(
            &format!("reading commit message from file {abs_path}"),
            &options.output,
        );
        options.strip_comments = true;
        console::verbose("removing comments from the commit message", &options.output);
        let message = read_file(file)?;
        handle_commit_message(&message, &options);
    } else if let Some(hash) = &args.hash {
        console::verbose("commit message source: hash", &options.output);
        let message = get_commit_message_from_hash(hash)?;
        handle_commit_message(&message, &options);
    } else if let Some(from_hash) = &args.from_hash {
        console::verbose("commit message source: hash range", &options.output);
        let messages = get_commit_messages_from_hash_range(from_hash, &args.to_hash)?;
        handle_multiple_commit_messages(&messages, &options);
    } else {
        unreachable!("invalid option is handled by clap");
    }

    Ok(())
}
