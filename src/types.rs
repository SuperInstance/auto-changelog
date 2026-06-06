use serde::{Deserialize, Serialize};

/// A parsed raw commit from git log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCommit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Types of conventional commits
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommitType {
    Feat,
    Fix,
    Refactor,
    Docs,
    Test,
    Chore,
    Build,
    Ci,
    Perf,
    Style,
    Revert,
    Breaking,
    Other(String),
}

impl std::fmt::Display for CommitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitType::Feat => write!(f, "feat"),
            CommitType::Fix => write!(f, "fix"),
            CommitType::Refactor => write!(f, "refactor"),
            CommitType::Docs => write!(f, "docs"),
            CommitType::Test => write!(f, "test"),
            CommitType::Chore => write!(f, "chore"),
            CommitType::Build => write!(f, "build"),
            CommitType::Ci => write!(f, "ci"),
            CommitType::Perf => write!(f, "perf"),
            CommitType::Style => write!(f, "style"),
            CommitType::Revert => write!(f, "revert"),
            CommitType::Breaking => write!(f, "breaking"),
            CommitType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// A classified commit with type, scope, and breaking flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedCommit {
    pub raw: RawCommit,
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub breaking_description: Option<String>,
}

/// A version group with its commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionGroup {
    pub version: String,
    pub date: String,
    pub commits: Vec<ClassifiedCommit>,
}

/// Result of conventional commit check
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub commit: RawCommit,
    pub valid: bool,
    pub reasons: Vec<String>,
}
