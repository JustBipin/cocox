use crate::constants::COMMIT_TYPES;
use crate::messages::{
    COMMIT_TYPE_MISSING_ERROR, DESCRIPTION_FULL_STOP_END_ERROR, DESCRIPTION_LINE_BREAK_ERROR,
    DESCRIPTION_MISSING_ERROR, DESCRIPTION_MULTIPLE_SPACE_START_ERROR,
    DESCRIPTION_NO_LEADING_SPACE_ERROR, INCORRECT_FORMAT_ERROR, SCOPE_EMPTY_ERROR,
    SCOPE_WHITESPACE_ERROR, SPACE_AFTER_COMMIT_TYPE_ERROR, SPACE_AFTER_SCOPE_ERROR,
    commit_type_invalid_error, header_length_error,
};
use regex::Regex;
use std::sync::LazyLock;

static SIMPLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let re_types = COMMIT_TYPES.join("|");
    let pattern = format!(
        r"(?s)^(?P<type>{re_types})(?P<scope>\(\S+\))?!?:(?: (?P<description>[^\s][^\n\r]+[^\.]))((\n\n(?P<body>.*))|(\s*))?$"
    );
    Regex::new(&pattern).unwrap()
});

static DETAILED_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)^(?P<type>\w+\s*)?(?:\((?P<scope>[^\)]*)\)(?P<space_after_scope>\s*))?!?(?P<colon>:\s?)?(?:(?P<description>[^\n\r]+))?(?P<body_separation>\n?\n?)(((?P<body>.*))|(\s*))?$",
    )
    .unwrap()
});

pub fn validate_header_length(message: &str, max: usize) -> Option<String> {
    let header = message.lines().next().unwrap_or("");
    if header.len() > max {
        Some(header_length_error(max))
    } else {
        None
    }
}

pub fn validate_simple_pattern(message: &str) -> Option<String> {
    if SIMPLE_PATTERN.is_match(message) {
        None
    } else {
        Some(INCORRECT_FORMAT_ERROR.to_string())
    }
}

pub fn validate_detailed_pattern(message: &str) -> Vec<String> {
    let Some(captures) = DETAILED_PATTERN.captures(message) else {
        return vec![INCORRECT_FORMAT_ERROR.to_string()];
    };

    if captures.name("colon").is_none() {
        return vec![INCORRECT_FORMAT_ERROR.to_string()];
    }

    [
        validate_commit_type(&captures),
        validate_commit_type_no_space_after(&captures),
        validate_scope(&captures),
        validate_scope_no_space_after(&captures),
        validate_description(&captures),
        validate_description_no_multiple_whitespace(&captures),
        validate_description_no_line_break(&captures),
        validate_description_no_full_stop_at_end(&captures),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn validate_commit_type(captures: &regex::Captures<'_>) -> Option<String> {
    match captures.name("type") {
        None => Some(COMMIT_TYPE_MISSING_ERROR.to_string()),
        Some(m) => {
            let commit_type = m.as_str().trim();
            if commit_type.is_empty() {
                Some(COMMIT_TYPE_MISSING_ERROR.to_string())
            } else if !COMMIT_TYPES.contains(&commit_type) {
                Some(commit_type_invalid_error(commit_type))
            } else {
                None
            }
        }
    }
}

fn validate_commit_type_no_space_after(captures: &regex::Captures<'_>) -> Option<String> {
    let commit_type = captures.name("type")?.as_str();
    commit_type
        .ends_with(' ')
        .then_some(SPACE_AFTER_COMMIT_TYPE_ERROR.to_string())
}

fn validate_scope(captures: &regex::Captures<'_>) -> Option<String> {
    let scope = captures.name("scope")?.as_str();
    if scope.is_empty() {
        Some(SCOPE_EMPTY_ERROR.to_string())
    } else if scope.contains(' ') {
        Some(SCOPE_WHITESPACE_ERROR.to_string())
    } else {
        None
    }
}

fn validate_scope_no_space_after(captures: &regex::Captures<'_>) -> Option<String> {
    let space_after_scope = captures.name("space_after_scope")?.as_str();
    space_after_scope
        .contains(' ')
        .then_some(SPACE_AFTER_SCOPE_ERROR.to_string())
}

fn validate_description(captures: &regex::Captures<'_>) -> Option<String> {
    let description = captures.name("description")?;
    if description.as_str().is_empty() {
        return Some(DESCRIPTION_MISSING_ERROR.to_string());
    }

    let colon = captures.name("colon")?.as_str();
    if !colon.ends_with(' ') {
        return Some(DESCRIPTION_NO_LEADING_SPACE_ERROR.to_string());
    }

    None
}

fn validate_description_no_multiple_whitespace(captures: &regex::Captures<'_>) -> Option<String> {
    let description = captures.name("description")?.as_str();
    description
        .starts_with(' ')
        .then_some(DESCRIPTION_MULTIPLE_SPACE_START_ERROR.to_string())
}

fn validate_description_no_line_break(captures: &regex::Captures<'_>) -> Option<String> {
    let body_separation = captures.name("body_separation")?.as_str();
    let body = captures.name("body").map(|m| m.as_str()).unwrap_or("");
    (body_separation == "\n" && !body.is_empty())
        .then_some(DESCRIPTION_LINE_BREAK_ERROR.to_string())
}

fn validate_description_no_full_stop_at_end(captures: &regex::Captures<'_>) -> Option<String> {
    let description = captures.name("description")?.as_str().trim();
    description
        .ends_with('.')
        .then_some(DESCRIPTION_FULL_STOP_END_ERROR.to_string())
}

pub fn run_validators(
    message: &str,
    skip_detail: bool,
    max_header_length: Option<usize>,
) -> (bool, Vec<String>) {
    // Only run header length check if a custom max was specified.
    if let Some(max) = max_header_length
        && let Some(error) = validate_header_length(message, max)
    {
        if skip_detail {
            return (false, vec![error]);
        }
        let mut errors = vec![error];
        errors.extend(validate_detailed_pattern(message));
        return (errors.is_empty(), errors);
    }

    if skip_detail {
        if let Some(error) = validate_simple_pattern(message) {
            return (false, vec![error]);
        }
        return (true, vec![]);
    }

    let errors = validate_detailed_pattern(message);
    (errors.is_empty(), errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::COMMIT_HEADER_MAX_LENGTH;

    #[test]
    fn simple_pattern_accepts_valid_commit() {
        assert!(validate_simple_pattern("feat: add new feature").is_none());
    }

    #[test]
    fn simple_pattern_rejects_invalid_commit() {
        assert_eq!(
            validate_simple_pattern("not a conventional commit"),
            Some(INCORRECT_FORMAT_ERROR.to_string())
        );
    }

    #[test]
    fn header_length_rejects_long_header() {
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH));
        assert_eq!(
            validate_header_length(&message, COMMIT_HEADER_MAX_LENGTH),
            Some(header_length_error(COMMIT_HEADER_MAX_LENGTH))
        );
    }

    #[test]
    fn header_length_passes_within_limit() {
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH - 10));
        assert!(validate_header_length(&message, COMMIT_HEADER_MAX_LENGTH).is_none());
    }

    #[test]
    fn skip_detail_returns_only_header_length_error() {
        let message = format!("Test {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH + 1));
        let (success, errors) = run_validators(&message, true, Some(COMMIT_HEADER_MAX_LENGTH));
        assert!(!success);
        assert_eq!(errors, vec![header_length_error(COMMIT_HEADER_MAX_LENGTH)]);
    }

    #[test]
    fn skip_detail_returns_only_incorrect_format_error() {
        let (success, errors) = run_validators("Test invalid commit message", true, None);
        assert!(!success);
        assert_eq!(errors, vec![INCORRECT_FORMAT_ERROR.to_string()]);
    }

    #[test]
    fn no_header_length_check_when_none() {
        // When max_header_length is None, long headers are accepted.
        let message = format!("feat: {}", "a".repeat(COMMIT_HEADER_MAX_LENGTH + 100));
        let (success, errors) = run_validators(&message, false, None);
        assert!(success);
        assert!(errors.is_empty());
    }

    #[test]
    fn custom_max_header_length_is_respected() {
        let message = "feat: this is exactly 25 characters long";
        // 39 chars, should fail with max=10
        let (success, errors) = run_validators(message, false, Some(10));
        assert!(!success);
        assert!(errors.contains(&header_length_error(10).to_string()));
    }

    #[test]
    fn detailed_pattern_reports_missing_type() {
        let errors = validate_detailed_pattern(": add new feature");
        assert!(errors.contains(&COMMIT_TYPE_MISSING_ERROR.to_string()));
    }

    #[test]
    fn detailed_pattern_reports_trailing_period() {
        let errors = validate_detailed_pattern("feat: add new feature.");
        assert!(errors.contains(&DESCRIPTION_FULL_STOP_END_ERROR.to_string()));
    }
}
