use crate::cli::Cli;
use crate::config::{Config, config, set_config, update_config};
use crate::console;
use crate::git_helpers::{get_commit_message_from_hash, get_commit_messages_from_hash_range};
use crate::linter::{LintOutcome, lint_commit_message_with_errors};
use crate::messages::{VALIDATION_FAILED, VALIDATION_SUCCESSFUL};
use crate::utils::remove_diff_from_commit_message;
use anyhow::{Context, Result};

impl Config {
    fn from_cli(args: &Cli) -> Self {
        Self {
            output: crate::config::OutputConfig::new(args.quiet, args.verbose),
            skip_detail: args.skip_detail,
            hide_input: args.hide_input,
            strip_comments: false,
            max_header_length: args.max_header_length,
        }
    }
}

fn read_file(file: &str) -> Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("failed to read commit message file `{}`", file))
        .map(|content| content.trim().to_string())
}

fn show_errors(message: &str, errors: &[String]) {
    let options = config();
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

fn handle_commit_message(message: &str) {
    let options = config();
    console::verbose("linting commit message:", &options.output);
    console::verbose(
        &format!("----------\n{message}\n----------"),
        &options.output,
    );

    let result = lint_commit_message_with_errors(message);

    match result.outcome {
        LintOutcome::Empty => std::process::exit(1),
        LintOutcome::Ignored => {
            console::verbose("commit message ignored, skipping lint", &options.output);
        }
        LintOutcome::Valid => {
            console::success(VALIDATION_SUCCESSFUL, &options.output);
        }
        LintOutcome::Invalid => {
            show_errors(message, &result.errors);
            std::process::exit(1);
        }
    }
}

fn handle_multiple_commit_messages(messages: &[String]) {
    let options = config();
    let mut has_error = false;

    for message in messages {
        let result = lint_commit_message_with_errors(message);

        match result.outcome {
            LintOutcome::Empty => std::process::exit(1),
            LintOutcome::Ignored | LintOutcome::Valid => {
                console::verbose("lint success", &options.output);
            }
            LintOutcome::Invalid => {
                has_error = true;
                show_errors(message, &result.errors);
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
    set_config(Config::from_cli(&args));

    console::verbose("starting cocox", &config().output);

    if let Some(message) = &args.message {
        console::verbose("commit message source: direct message", &config().output);
        handle_commit_message(message.trim());
    } else if let Some(file) = &args.file {
        console::verbose("commit message source: file", &config().output);
        let abs_path = std::fs::canonicalize(file)
            .unwrap_or_else(|_| std::path::PathBuf::from(file))
            .display()
            .to_string();
        console::verbose(
            &format!("reading commit message from file {abs_path}"),
            &config().output,
        );
        update_config(|config| config.strip_comments = true);
        console::verbose(
            "removing comments from the commit message",
            &config().output,
        );
        let message = read_file(file)?;
        handle_commit_message(&message);
    } else if let Some(hash) = &args.hash {
        console::verbose("commit message source: hash", &config().output);
        let message = get_commit_message_from_hash(hash)?;
        handle_commit_message(&message);
    } else if let Some(from_hash) = &args.from_hash {
        console::verbose("commit message source: hash range", &config().output);
        let messages = get_commit_messages_from_hash_range(from_hash, &args.to_hash)?;
        handle_multiple_commit_messages(&messages);
    } else {
        unreachable!("invalid option is handled by clap");
    }

    Ok(())
}
