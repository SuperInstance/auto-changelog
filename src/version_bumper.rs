use crate::types::{ClassifiedCommit, CommitType, VersionGroup};
use crate::git_parser::GitParser;

/// Determines next semver version based on commit types
pub struct VersionBumper;

impl VersionBumper {
    pub fn new() -> Self {
        Self
    }

    /// Get the current version from the latest tag
    pub fn current_version(&self, parser: &GitParser) -> Option<String> {
        parser.last_tag()
    }

    /// Calculate the next version based on classified commits
    pub fn bump(&self, commits: &[ClassifiedCommit], current: Option<&str>) -> String {
        let (mut major, mut minor, mut patch) = match current {
            Some(v) => self.parse_version(v),
            None => (0, 0, 0),
        };

        let has_breaking = commits.iter().any(|c| c.breaking);
        let has_feat = commits.iter().any(|c| c.commit_type == CommitType::Feat);

        if has_breaking {
            major += 1;
            minor = 0;
            patch = 0;
        } else if has_feat {
            minor += 1;
            patch = 0;
        } else {
            patch += 1;
        }

        format!("{major}.{minor}.{patch}")
    }

    /// Group commits by version (using tags as boundaries)
    pub fn group_by_version(
        &self,
        commits: &[ClassifiedCommit],
        parser: &GitParser,
    ) -> Vec<VersionGroup> {
        let tags = parser.tags();

        if tags.is_empty() {
            // No tags — everything is "Unreleased"
            let version = if commits.is_empty() {
                "0.1.0".to_string()
            } else {
                self.bump(commits, None)
            };
            return vec![VersionGroup {
                version,
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                commits: commits.to_vec(),
            }];
        }

        // For simplicity, group all since-last-tag as upcoming version
        let last_tag = &tags[0];
        let next_version = self.bump(commits, Some(last_tag));
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        vec![VersionGroup {
            version: next_version,
            date,
            commits: commits.to_vec(),
        }]
    }

    fn parse_version(&self, tag: &str) -> (u32, u32, u32) {
        let cleaned = tag.trim_start_matches('v');
        let parts: Vec<&str> = cleaned.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RawCommit;

    fn make_classified(commit_type: CommitType, breaking: bool) -> ClassifiedCommit {
        ClassifiedCommit {
            raw: RawCommit {
                hash: "abc".to_string(),
                author: "test".to_string(),
                date: "2024-01-01".to_string(),
                message: "test".to_string(),
            },
            commit_type,
            scope: None,
            description: "test".to_string(),
            body: None,
            breaking,
            breaking_description: None,
        }
    }

    #[test]
    fn test_patch_bump() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Fix, false)];
        assert_eq!(bumper.bump(&commits, Some("1.0.0")), "1.0.1");
    }

    #[test]
    fn test_minor_bump_on_feat() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Feat, false)];
        assert_eq!(bumper.bump(&commits, Some("1.0.0")), "1.1.0");
    }

    #[test]
    fn test_major_bump_on_breaking() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Feat, true)];
        assert_eq!(bumper.bump(&commits, Some("1.0.0")), "2.0.0");
    }

    #[test]
    fn test_initial_version() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Feat, false)];
        assert_eq!(bumper.bump(&commits, None), "0.1.0");
    }

    #[test]
    fn test_breaking_with_v_prefix() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Fix, true)];
        assert_eq!(bumper.bump(&commits, Some("v2.3.4")), "3.0.0");
    }

    #[test]
    fn test_chore_is_patch() {
        let bumper = VersionBumper::new();
        let commits = vec![make_classified(CommitType::Chore, false)];
        assert_eq!(bumper.bump(&commits, Some("1.2.3")), "1.2.4");
    }

    #[test]
    fn test_breaking_overrides_feat() {
        let bumper = VersionBumper::new();
        let commits = vec![
            make_classified(CommitType::Feat, false),
            make_classified(CommitType::Fix, true),
        ];
        assert_eq!(bumper.bump(&commits, Some("1.0.0")), "2.0.0");
    }
}
