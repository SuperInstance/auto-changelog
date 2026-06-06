use crate::types::RawCommit;
use std::process::Command;

/// Parses git log output into structured commits
pub struct GitParser {
    repo_path: String,
}

impl GitParser {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
        }
    }

    /// Parse git log into raw commits
    /// `depth` controls how many tags back to go (None = all history)
    pub fn parse_log(&self, depth: Option<usize>) -> Result<Vec<RawCommit>, String> {
        let range = match depth {
            Some(_) => self.last_tag_range()?,
            None => "--all".to_string(),
        };

        let output = Command::new("git")
            .args(["log", &range, "--pretty=format:%H%n%an%n%aI%n%B%n---COMMIT_END---"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("Failed to run git log: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git log failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_output(&stdout))
    }

    /// Get the last tag for version detection
    pub fn last_tag(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .current_dir(&self.repo_path)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Get all tags sorted by version
    pub fn tags(&self) -> Vec<String> {
        let output = match Command::new("git")
            .args(["tag", "--sort=-version:refname"])
            .current_dir(&self.repo_path)
            .output()
        {
            Ok(o) => o,
            Err(_) => return vec![],
        };

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        } else {
            vec![]
        }
    }

    /// Get date for a specific tag
    pub fn tag_date(&self, tag: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%aI", tag])
            .current_dir(&self.repo_path)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    fn last_tag_range(&self) -> Result<String, String> {
        let tag = self.last_tag();
        match tag {
            Some(t) => Ok(format!("{t}..HEAD")),
            None => Ok("HEAD".to_string()),
        }
    }

    fn parse_output(&self, output: &str) -> Vec<RawCommit> {
        output
            .split("---COMMIT_END---")
            .filter_map(|block| {
                let lines: Vec<&str> = block.lines().collect();
                if lines.len() < 3 {
                    return None;
                }
                let hash = lines.first()?.trim().to_string();
                let author = lines.get(1)?.trim().to_string();
                let date = lines.get(2)?.trim().to_string();
                let message = lines[3..].join("\n").trim().to_string();
                if hash.is_empty() {
                    return None;
                }
                Some(RawCommit {
                    hash,
                    author,
                    date,
                    message,
                })
            })
            .collect()
    }
}
