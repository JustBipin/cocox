use crate::constants::COMMIT_TYPES;
use crate::utils::{is_empty, is_ignored};
use regex::Regex;
use std::sync::LazyLock;

static LINT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let re_types = COMMIT_TYPES.join("|");

    let pattern = format!(
        r"(?s)^(?P<type>{})(?P<scope>\(\S+\))?!?:(?: (?P<description>[^\s][^\n\r]+[^\.]))((\n\n(?P<body>.*))|(\s*))?$",
        re_types
    );

    Regex::new(&pattern).unwrap()
});

#[derive(Debug, PartialEq, Eq)]
pub enum LintOutcome {
    Valid,
    Invalid,
    Ignored,
    Empty,
}

/// Evaluates a commit message, matches the conventional commit format
///  and returns its linting outcome.
pub fn lint_commit_message(message: &str) -> LintOutcome {
    println!(
        "========= Linting Message:: {} ===========",
        message.lines().next().unwrap_or("")
    );
    if is_empty(message) {
        LintOutcome::Empty
    } else if is_ignored(message) {
        LintOutcome::Ignored
    } else if validate_message(message) {
        LintOutcome::Valid
    } else {
        LintOutcome::Invalid
    }
}

fn validate_message(message: &str) -> bool {
    LINT_REGEX.is_match(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_basic_conventional_commit() {
        assert!(validate_message("feat: add new feature"));
    }

    #[test]
    fn accepts_every_known_commit_type() {
        for kind in COMMIT_TYPES {
            assert!(validate_message(&format!("{}: do the thing", kind)));
        }
    }

    #[test]
    fn accepts_commit_with_scope() {
        assert!(validate_message("feat(parser): add new feature"));
        assert!(validate_message(
            "build(deps-dev): bump @babel/traverse from 7.22.17 to 7.24.0"
        ));
    }

    #[test]
    fn accepts_breaking_change_marker() {
        assert!(validate_message("feat!: breaking feature"));
        assert!(validate_message("feat(api)!: breaking feature"));
    }

    #[test]
    fn accepts_body_separated_by_blank_line() {
        assert!(validate_message("feat: add new feature\n\nthis is body"));
        assert!(validate_message(
            "feat: add new feature\n\nthis is body\n\ntest"
        ));
    }

    #[test]
    fn accepts_trailing_newline() {
        assert!(validate_message("feat: add new feature\n"));
    }

    #[test]
    fn rejects_empty_message() {
        assert!(!validate_message(""));
    }

    #[test]
    fn rejects_missing_colon() {
        assert!(!validate_message("feat add new feature"));
    }

    #[test]
    fn rejects_missing_type() {
        assert!(!validate_message(": add new feature"));
        assert!(!validate_message("(invalid): add new feature"));
    }

    #[test]
    fn rejects_unknown_type() {
        assert!(!validate_message("invalid: add new feature"));
        assert!(!validate_message("foo(bar): add new feature"));
    }

    #[test]
    fn rejects_space_between_type_and_scope() {
        assert!(!validate_message("feat (test): add new feature"));
    }

    #[test]
    fn rejects_empty_or_whitespace_scope() {
        assert!(!validate_message("feat(): add new feature"));
        assert!(!validate_message("feat( ): add new feature"));
        assert!(!validate_message("feat(hello world): add new feature"));
    }

    #[test]
    fn rejects_space_between_scope_and_colon() {
        assert!(!validate_message("feat(test) : add new feature"));
    }

    #[test]
    fn rejects_description_without_leading_space() {
        assert!(!validate_message("feat:add new feature"));
    }

    #[test]
    fn rejects_description_with_extra_leading_space() {
        assert!(!validate_message("feat:  add new feature"));
    }

    #[test]
    fn rejects_line_break_inside_description() {
        assert!(!validate_message("feat: add new feature\nhello baby"));
    }

    #[test]
    fn rejects_missing_description() {
        assert!(!validate_message("feat(test):"));
        assert!(!validate_message("feat(test): "));
    }

    #[test]
    fn rejects_description_with_trailing_period() {
        assert!(!validate_message("feat(test): add new feature."));
    }

    #[test]
    fn rejects_ignore_style_messages() {
        // The linter itself does not know about ignore patterns; those are
        // short-circuited one layer up in `lint_commit_message` via
        // `is_ignored`. The regex below should therefore reject these.
        assert!(!validate_message("Merge pull request #123"));
        assert!(!validate_message("Bump urllib3 from 1.26.5 to 1.26.17"));
        assert!(!validate_message("Initial commit"));
    }

    // ---- lint_commit_message tests ------------------------------------------

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
        assert_eq!(
            lint_commit_message("Revert \"feat: add something\""),
            LintOutcome::Ignored
        );
    }

    #[test]
    fn returns_outcome_valid() {
        assert_eq!(
            lint_commit_message("feat: add new feature"),
            LintOutcome::Valid
        );
        assert_eq!(
            lint_commit_message("fix(parser): handle empty input"),
            LintOutcome::Valid
        );
    }

    #[test]
    fn returns_outcome_invalid() {
        assert_eq!(
            lint_commit_message("not a conventional commit"),
            LintOutcome::Invalid
        );
        assert_eq!(
            lint_commit_message("feat: trailing period."),
            LintOutcome::Invalid
        );
    }
}
