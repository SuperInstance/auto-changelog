use crate::types::{CheckResult, RawCommit};
use regex::Regex;

/// Validates commits follow conventional commit format
pub struct ConventionalChecker {
    pattern: Regex,
}

impl ConventionalChecker {
    pub fn new() -> Self {
        Self {
            // type(scope)!: description
            pattern: Regex::new(
                r"^(feat|fix|refactor|docs|test|chore|build|ci|perf|style|revert)(\([^)]+\))?(!)?:\s*.+"
            ).unwrap(),
        }
    }

    /// Check a list of commits for conventional compliance
    pub fn check(&self, commits: &[RawCommit]) -> Vec<CheckResult> {
        commits.iter().map(|c| self.check_one(c)).collect()
    }

    fn check_one(&self, commit: &RawCommit) -> CheckResult {
        let first_line = commit.message.lines().next().unwrap_or("");
        let mut reasons = Vec::new();

        if first_line.is_empty() {
            reasons.push("Commit message is empty".to_string());
            return CheckResult {
                commit: commit.clone(),
                valid: false,
                reasons,
            };
        }

        if !self.pattern.is_match(first_line) {
            // Specific checks for better error messages
            if !first_line.contains(':') {
                reasons.push(
                    "Missing colon separator — format should be `type: description`".to_string(),
                );
            } else {
                let before_colon = first_line.split(':').next().unwrap_or("");
                let type_part = before_colon.split('(').next().unwrap_or("").trim();
                if ![
                    "feat", "fix", "refactor", "docs", "test", "chore", "build", "ci", "perf",
                    "style", "revert",
                ]
                .contains(&type_part)
                {
                    reasons.push(format!(
                        "Unknown commit type '{type_part}' — use feat|fix|refactor|docs|test|chore|build|ci|perf|style|revert"
                    ));
                }
            }

            if !reasons.is_empty() {
                // Already have specific reasons
            } else {
                reasons.push("Does not match conventional commit format: `type(scope)!: description`".to_string());
            }
        }

        // Check subject line length
        if first_line.len() > 72 {
            reasons.push(format!(
                "Subject line is {} characters (max 72)",
                first_line.len()
            ));
        }

        CheckResult {
            commit: commit.clone(),
            valid: reasons.is_empty(),
            reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(msg: &str) -> RawCommit {
        RawCommit {
            hash: "abc123".to_string(),
            author: "test".to_string(),
            date: "2024-01-01".to_string(),
            message: msg.to_string(),
        }
    }

    #[test]
    fn test_valid_feat() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("feat: add feature")]);
        assert!(results[0].valid);
    }

    #[test]
    fn test_valid_with_scope() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("fix(api): handle null response")]);
        assert!(results[0].valid);
    }

    #[test]
    fn test_valid_breaking() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("feat!: new API")]);
        assert!(results[0].valid);
    }

    #[test]
    fn test_invalid_no_type() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("random message")]);
        assert!(!results[0].valid);
        assert!(!results[0].reasons.is_empty());
    }

    #[test]
    fn test_invalid_no_colon() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("feat added something")]);
        assert!(!results[0].valid);
    }

    #[test]
    fn test_unknown_type() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("magic: do something")]);
        assert!(!results[0].valid);
        assert!(results[0].reasons[0].contains("Unknown commit type"));
    }

    #[test]
    fn test_empty_message() {
        let checker = ConventionalChecker::new();
        let results = checker.check(&[make_raw("")]);
        assert!(!results[0].valid);
        assert!(results[0].reasons[0].contains("empty"));
    }
}
