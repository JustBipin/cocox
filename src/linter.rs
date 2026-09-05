use crate::config::config;
use crate::utils::{is_empty, is_ignored, remove_comments};
use crate::validators::run_validators;

#[derive(Debug, PartialEq, Eq)]
pub struct LintResult {
    pub outcome: LintOutcome,
    pub errors: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LintOutcome {
    Valid,
    Invalid,
    Ignored,
    Empty,
}

/// Evaluates a commit message and returns its linting outcome.
pub fn lint_commit_message(message: &str) -> LintOutcome {
    lint_commit_message_with_errors(message).outcome
}

pub fn lint_commit_message_with_errors(message: &str) -> LintResult {
    let options = config();
    let mut message = message.to_string();

    if options.strip_comments {
        message = remove_comments(&message);
    }

    if is_empty(&message) {
        return LintResult {
            outcome: LintOutcome::Empty,
            errors: vec![],
        };
    }

    if is_ignored(&message) {
        return LintResult {
            outcome: LintOutcome::Ignored,
            errors: vec![],
        };
    }

    let (success, errors) =
        run_validators(&message, options.skip_detail, options.max_header_length);
    if success {
        LintResult {
            outcome: LintOutcome::Valid,
            errors: vec![],
        }
    } else {
        LintResult {
            outcome: LintOutcome::Invalid,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ConfigGuard};
    use crate::constants::COMMIT_TYPES;

    #[test]
    fn accepts_basic_conventional_commit() {
        assert_eq!(
            lint_commit_message("feat: add new feature"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn accepts_every_known_commit_type() {
        for kind in COMMIT_TYPES {
            assert_eq!(
                lint_commit_message(&format!("{}: do the thing", kind)),
                LintOutcome::Valid
            );
        }
    }

    #[test]
    fn accepts_commit_with_scope() {
        assert_eq!(
            lint_commit_message("feat(parser): add new feature"),
            LintOutcome::Valid
        );
        assert_eq!(
            lint_commit_message("build(deps-dev): bump @babel/traverse from 7.22.17 to 7.24.0"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn accepts_breaking_change_marker() {
        assert_eq!(
            lint_commit_message("feat!: breaking feature"),
            LintOutcome::Valid
        );
        assert_eq!(
            lint_commit_message("feat(api)!: breaking feature"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn accepts_body_separated_by_blank_line() {
        assert_eq!(
            lint_commit_message("feat: add new feature\n\nthis is body"),
            LintOutcome::Valid
        );
        assert_eq!(
            lint_commit_message("feat: add new feature\n\nthis is body\n\ntest"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn accepts_trailing_newline() {
        assert_eq!(
            lint_commit_message("feat: add new feature\n"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn rejects_empty_message() {
        assert_eq!(lint_commit_message(""), LintOutcome::Empty);
    }

    #[test]
    fn rejects_missing_colon() {
        assert_eq!(
            lint_commit_message("feat add new feature"),
            LintOutcome::Invalid
        );
    }

    #[test]
    fn rejects_unknown_type() {
        assert_eq!(
            lint_commit_message("invalid: add new feature"),
            LintOutcome::Invalid
        );
    }

    #[test]
    fn rejects_description_without_leading_space() {
        assert_eq!(
            lint_commit_message("feat:add new feature"),
            LintOutcome::Invalid
        );
    }

    #[test]
    fn rejects_description_with_trailing_period() {
        assert_eq!(
            lint_commit_message("feat: trailing period."),
            LintOutcome::Invalid
        );
    }

    #[test]
    fn strip_comments_allows_valid_message_with_git_comments() {
        let _guard = ConfigGuard::set(Config {
            strip_comments: true,
            ..Default::default()
        });
        let message = "feat(scope): add new feature\n#this is a comment";
        let result = lint_commit_message_with_errors(message);
        assert_eq!(result.outcome, LintOutcome::Valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn returns_outcome_empty() {
        assert_eq!(lint_commit_message(""), LintOutcome::Empty);
        assert_eq!(lint_commit_message("   "), LintOutcome::Empty);
        assert_eq!(lint_commit_message("\n\n"), LintOutcome::Empty);
    }

    #[test]
    fn returns_outcome_ignored() {
        assert_eq!(
            lint_commit_message("Merge branch 'main' into develop"),
            LintOutcome::Ignored
        );
        assert_eq!(lint_commit_message("Initial commit"), LintOutcome::Ignored);
    }

    #[test]
    fn returns_outcome_invalid() {
        assert_eq!(
            lint_commit_message("not a conventional commit"),
            LintOutcome::Invalid
        );
    }
}
