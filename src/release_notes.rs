use crate::types::VersionGroup;

/// Generates release notes for the latest version
pub struct ReleaseNotes;

impl ReleaseNotes {
    pub fn new() -> Self {
        Self
    }

    /// Generate release notes markdown from version groups (latest only)
    pub fn generate(&self, groups: &[VersionGroup]) -> String {
        let group = match groups.first() {
            Some(g) => g,
            None => return "No changes found.".to_string(),
        };

        let mut md = String::new();
        md.push_str(&format!(
            "# Release Notes - v{}\n\n",
            group.version
        ));
        md.push_str(&format!("**Date:** {}\n\n", group.date));

        // Summary
        let feat_count = group
            .commits
            .iter()
            .filter(|c| c.commit_type == crate::types::CommitType::Feat)
            .count();
        let fix_count = group
            .commits
            .iter()
            .filter(|c| c.commit_type == crate::types::CommitType::Fix)
            .count();
        let breaking_count = group.commits.iter().filter(|c| c.breaking).count();

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "This release includes {} changes",
            group.commits.len()
        ));
        if feat_count > 0 {
            md.push_str(&format!(", {feat_count} new features"));
        }
        if fix_count > 0 {
            md.push_str(&format!(", {fix_count} bug fixes"));
        }
        if breaking_count > 0 {
            md.push_str(&format!(
                ", and {breaking_count} breaking change(s)"
            ));
        }
        md.push_str(".\n\n");

        // Breaking changes warning
        if breaking_count > 0 {
            md.push_str("## ⚠ Breaking Changes\n\n");
            for commit in &group.commits {
                if commit.breaking {
                    let desc = commit
                        .breaking_description
                        .as_deref()
                        .unwrap_or(&commit.description);
                    md.push_str(&format!("- **{desc}**\n"));
                }
            }
            md.push('\n');
        }

        // Highlights
        md.push_str("## Highlights\n\n");
        for commit in &group.commits {
            let icon = match commit.commit_type {
                crate::types::CommitType::Feat => "✨",
                crate::types::CommitType::Fix => "🐛",
                crate::types::CommitType::Refactor => "♻️",
                crate::types::CommitType::Perf => "⚡",
                crate::types::CommitType::Docs => "📚",
                crate::types::CommitType::Test => "🧪",
                _ => "📝",
            };
            let scope = commit
                .scope
                .as_deref()
                .map(|s| format!("[{s}] "))
                .unwrap_or_default();
            md.push_str(&format!(
                "- {icon} **{scope}{}** ({}…)\n",
                commit.description,
                &commit.raw.hash[..7]
            ));
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClassifiedCommit, CommitType, RawCommit};

    fn make_group(version: &str, commits: Vec<ClassifiedCommit>) -> VersionGroup {
        VersionGroup {
            version: version.to_string(),
            date: "2024-06-01".to_string(),
            commits,
        }
    }

    fn make_classified(ct: CommitType, desc: &str) -> ClassifiedCommit {
        ClassifiedCommit {
            raw: RawCommit {
                hash: "abc1234def".to_string(),
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
        let rn = ReleaseNotes::new();
        assert_eq!(rn.generate(&[]), "No changes found.");
    }

    #[test]
    fn test_release_notes_content() {
        let rn = ReleaseNotes::new();
        let groups = vec![make_group(
            "1.2.0",
            vec![
                make_classified(CommitType::Feat, "add search"),
                make_classified(CommitType::Fix, "fix login"),
            ],
        )];
        let md = rn.generate(&groups);
        assert!(md.contains("# Release Notes - v1.2.0"));
        assert!(md.contains("1 new features"));
        assert!(md.contains("1 bug fixes"));
        assert!(md.contains("add search"));
    }
}
