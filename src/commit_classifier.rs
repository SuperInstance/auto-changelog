use crate::types::{ClassifiedCommit, CommitType, RawCommit};
use regex::Regex;

/// Classifies commits by parsing conventional commit format
pub struct CommitClassifier {
    pattern: Regex,
    breaking_pattern: Regex,
}

impl CommitClassifier {
    pub fn new() -> Self {
        Self {
            // type(scope)!: description
            pattern: Regex::new(
                r"^(?P<type>[a-z]+)(\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s*(?P<desc>.+)"
            ).unwrap(),
            breaking_pattern: Regex::new(r"(?i)BREAKING[ -]CHANGE:\s*(.+)").unwrap(),
        }
    }

    /// Classify a single raw commit
    pub fn classify(&self, raw: RawCommit) -> ClassifiedCommit {
        let first_line = raw.message.lines().next().unwrap_or("").to_string();
        let body: String = raw.message.lines().skip(1).collect::<Vec<_>>().join("\n");

        if let Some(caps) = self.pattern.captures(&first_line) {
            let type_str = caps.name("type").map(|m| m.as_str()).unwrap_or("");
            let scope = caps.name("scope").map(|m| m.as_str().to_string());
            let has_bang = caps.name("breaking").is_some();
            let description = caps
                .name("desc")
                .map(|m| m.as_str().to_string())
                .unwrap_or(first_line.clone());

            let breaking_in_body = self.breaking_pattern.captures(&body);
            let is_breaking = has_bang || breaking_in_body.is_some();
            let breaking_desc = breaking_in_body
                .map(|c| c[1].trim().to_string())
                .or_else(|| {
                    if has_bang {
                        Some(description.clone())
                    } else {
                        None
                    }
                });

            ClassifiedCommit {
                commit_type: Self::parse_type(type_str),
                raw,
                scope,
                description,
                body: if body.trim().is_empty() {
                    None
                } else {
                    Some(body.trim().to_string())
                },
                breaking: is_breaking,
                breaking_description: breaking_desc,
            }
        } else {
            // Not a conventional commit - classify as Other
            ClassifiedCommit {
                commit_type: CommitType::Other("misc".to_string()),
                raw,
                scope: None,
                description: first_line,
                body: if body.trim().is_empty() {
                    None
                } else {
                    Some(body.trim().to_string())
                },
                breaking: false,
                breaking_description: None,
            }
        }
    }

    /// Classify multiple commits
    pub fn classify_all(&self, raw_commits: Vec<RawCommit>) -> Vec<ClassifiedCommit> {
        raw_commits.into_iter().map(|c| self.classify(c)).collect()
    }

    fn parse_type(s: &str) -> CommitType {
        match s {
            "feat" => CommitType::Feat,
            "fix" => CommitType::Fix,
            "refactor" => CommitType::Refactor,
            "docs" => CommitType::Docs,
            "test" => CommitType::Test,
            "chore" => CommitType::Chore,
            "build" => CommitType::Build,
            "ci" => CommitType::Ci,
            "perf" => CommitType::Perf,
            "style" => CommitType::Style,
            "revert" => CommitType::Revert,
            other => CommitType::Other(other.to_string()),
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
    fn test_feat_commit() {
        let classifier = CommitClassifier::new();
        let result = classifier.classify(make_raw("feat(auth): add login support"));
        assert_eq!(result.commit_type, CommitType::Feat);
        assert_eq!(result.scope, Some("auth".to_string()));
        assert_eq!(result.description, "add login support");
        assert!(!result.breaking);
    }

    #[test]
    fn test_fix_commit_no_scope() {
        let classifier = CommitClassifier::new();
        let result = classifier.classify(make_raw("fix: resolve null pointer"));
        assert_eq!(result.commit_type, CommitType::Fix);
        assert_eq!(result.scope, None);
        assert_eq!(result.description, "resolve null pointer");
    }

    #[test]
    fn test_breaking_bang() {
        let classifier = CommitClassifier::new();
        let result = classifier.classify(make_raw("feat(api)!: redesign endpoints"));
        assert!(result.breaking);
        assert_eq!(result.breaking_description, Some("redesign endpoints".to_string()));
    }

    #[test]
    fn test_breaking_in_body() {
        let classifier = CommitClassifier::new();
        let msg = "feat: new thing\n\nBREAKING CHANGE: old API removed";
        let result = classifier.classify(make_raw(msg));
        assert!(result.breaking);
        assert_eq!(
            result.breaking_description,
            Some("old API removed".to_string())
        );
    }

    #[test]
    fn test_non_conventional() {
        let classifier = CommitClassifier::new();
        let result = classifier.classify(make_raw("random commit message"));
        assert_eq!(result.commit_type, CommitType::Other("misc".to_string()));
        assert_eq!(result.description, "random commit message");
    }

    #[test]
    fn test_all_types() {
        let classifier = CommitClassifier::new();
        for (input, expected) in [
            ("feat: a", CommitType::Feat),
            ("fix: b", CommitType::Fix),
            ("refactor: c", CommitType::Refactor),
            ("docs: d", CommitType::Docs),
            ("test: e", CommitType::Test),
            ("chore: f", CommitType::Chore),
            ("build: g", CommitType::Build),
            ("ci: h", CommitType::Ci),
            ("perf: i", CommitType::Perf),
            ("style: j", CommitType::Style),
            ("revert: k", CommitType::Revert),
        ] {
            let result = classifier.classify(make_raw(input));
            assert_eq!(result.commit_type, expected, "Failed for: {input}");
        }
    }

    #[test]
    fn test_classify_all() {
        let classifier = CommitClassifier::new();
        let commits = vec![
            make_raw("feat: one"),
            make_raw("fix: two"),
            make_raw("chore: three"),
        ];
        let results = classifier.classify_all(commits);
        assert_eq!(results.len(), 3);
    }
}
