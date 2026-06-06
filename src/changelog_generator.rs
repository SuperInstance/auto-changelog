use crate::types::{CommitType, VersionGroup};

/// Generates markdown changelog from version groups
pub struct ChangelogGenerator;

impl ChangelogGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a full markdown changelog
    pub fn generate(&self, groups: &[VersionGroup]) -> String {
        let mut md = String::new();
        md.push_str("# Changelog\n\n");
        md.push_str("All notable changes to this project will be documented in this file.\n\n");

        for group in groups {
            md.push_str(&format!("## [{}] - {}\n\n", group.version, group.date));

            // Breaking changes first
            let breaking: Vec<_> = group.commits.iter().filter(|c| c.breaking).collect();
            if !breaking.is_empty() {
                md.push_str("### ⚠ BREAKING CHANGES\n\n");
                for commit in &breaking {
                    let desc = commit
                        .breaking_description
                        .as_deref()
                        .unwrap_or(&commit.description);
                    md.push_str(&format!(
                        "- {} ({})\n",
                        desc,
                        &commit.raw.hash[..7]
                    ));
                }
                md.push('\n');
            }

            // Group by type
            let sections = [
                (CommitType::Feat, "✨ Features"),
                (CommitType::Fix, "🐛 Bug Fixes"),
                (CommitType::Refactor, "♻️ Refactor"),
                (CommitType::Perf, "⚡ Performance"),
                (CommitType::Docs, "📚 Documentation"),
                (CommitType::Test, "🧪 Tests"),
                (CommitType::Build, "📦 Build"),
                (CommitType::Ci, "👷 CI"),
                (CommitType::Chore, "🔧 Chore"),
                (CommitType::Style, "💄 Style"),
                (CommitType::Revert, "⏪ Revert"),
            ];

            for (ct, label) in &sections {
                let matching: Vec<_> = group
                    .commits
                    .iter()
                    .filter(|c| c.commit_type == *ct && !c.breaking)
                    .collect();
                if !matching.is_empty() {
                    md.push_str(&format!("### {label}\n\n"));
                    for commit in &matching {
                        let scope = commit
                            .scope
                            .as_deref()
                            .map(|s| format!("**{s}**: "))
                            .unwrap_or_default();
                        md.push_str(&format!(
                            "- {}{} ({})\n",
                            scope,
                            commit.description,
                            &commit.raw.hash[..7]
                        ));
                    }
                    md.push('\n');
                }
            }

            // Other/misc commits
            let other: Vec<_> = group
                .commits
                .iter()
                .filter(|c| matches!(c.commit_type, CommitType::Other(_)) && !c.breaking)
                .collect();
            if !other.is_empty() {
                md.push_str("### 📝 Other Changes\n\n");
                for commit in &other {
                    md.push_str(&format!(
                        "- {} ({})\n",
                        commit.description,
                        &commit.raw.hash[..7]
                    ));
                }
                md.push('\n');
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClassifiedCommit, RawCommit};

    fn make_group(version: &str, commits: Vec<ClassifiedCommit>) -> VersionGroup {
        VersionGroup {
            version: version.to_string(),
            date: "2024-06-01".to_string(),
            commits,
        }
    }

    fn make_classified(ct: CommitType, desc: &str, hash: &str) -> ClassifiedCommit {
        ClassifiedCommit {
            raw: RawCommit {
                hash: hash.to_string(),
                author: "test".to_string(),
                date: "2024-01-01".to_string(),
                message: desc.to_string(),
            },
            commit_type: ct,
            scope: None,
            description: desc.to_string(),
            body: None,
            breaking: false,
            breaking_description: None,
        }
    }

    #[test]
    fn test_empty_groups() {
        let gen = ChangelogGenerator::new();
        let md = gen.generate(&[]);
        assert!(md.contains("# Changelog"));
    }

    #[test]
    fn test_single_version() {
        let gen = ChangelogGenerator::new();
        let groups = vec![make_group(
            "1.1.0",
            vec![
                make_classified(CommitType::Feat, "new feature", "aaa1111"),
                make_classified(CommitType::Fix, "bug fix", "bbb2222"),
            ],
        )];
        let md = gen.generate(&groups);
        assert!(md.contains("## [1.1.0]"));
        assert!(md.contains("✨ Features"));
        assert!(md.contains("new feature"));
        assert!(md.contains("🐛 Bug Fixes"));
        assert!(md.contains("bug fix"));
    }

    #[test]
    fn test_breaking_changes_section() {
        let gen = ChangelogGenerator::new();
        let mut c = make_classified(CommitType::Feat, "redesign", "ccc3333");
        c.breaking = true;
        c.breaking_description = Some("old API removed".to_string());
        let groups = vec![make_group("2.0.0", vec![c])];
        let md = gen.generate(&groups);
        assert!(md.contains("⚠ BREAKING CHANGES"));
        assert!(md.contains("old API removed"));
    }
}
