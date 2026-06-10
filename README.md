# auto-changelog

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)]()
[![SuperInstance](https://img.shields.io/badge/part%20of-SuperInstance-9cf.svg)](https://github.com/SuperInstance)

Automatic changelog generator from conventional commits. Parses `git log`, classifies commits by [Conventional Commits](https://www.conventionalcommits.org/) format, computes semver bumps, and outputs grouped markdown changelogs and release notes.

## Overview

Every repo in the SuperInstance ecosystem follows conventional commits. `auto-changelog` turns those commit messages into structured changelogs without any manual input. It reads your git history, classifies each commit, determines the next version number, and generates markdown.

No config files. No templates. Just run it against any repo that uses conventional commits.

## Installation

```bash
cargo install --path .
```

Dependencies: `clap` (CLI), `regex` (parsing), `semver`, `chrono`, `serde`/`serde_json` (types).

## Usage

```bash
# Generate full changelog to stdout
auto-changelog changelog

# Write changelog to a file
auto-changelog changelog --output CHANGELOG.md

# Include all history (not just since last tag)
auto-changelog changelog --all

# What's the next version?
auto-changelog next-version

# Generate release notes for the latest version
auto-changelog release-notes --output RELEASE.md

# Check last 20 commits for conventional compliance
auto-changelog check

# Check last 50 commits
auto-changelog check --count 50
```

Point at a different repo with `-r`:

```bash
auto-changelog -r /path/to/repo changelog
```

## Architecture

```
┌──────────────────────────────────────────────┐
│                   main.rs                     │
│            CLI (clap Parser)                  │
├──────────────────────────────────────────────┤
│ git_parser.rs    → RawCommit                  │
│    parse_log(), last_tag(), tags()            │
├──────────────────────────────────────────────┤
│ commit_classifier.rs → ClassifiedCommit       │
│    classify(), classify_all()                 │
├──────────────────────────────────────────────┤
│ version_bumper.rs  → VersionGroup             │
│    bump(), group_by_version()                 │
├──────────────────────────────────────────────┤
│ changelog_generator.rs → Markdown string      │
│    generate()                                 │
├──────────────────────────────────────────────┤
│ release_notes.rs → Release notes markdown     │
│    generate()                                 │
├──────────────────────────────────────────────┤
│ conventional_checker.rs → CheckResult[]       │
│    check()                                    │
└──────────────────────────────────────────────┘
```

## API Reference

### Types (`types.rs`)

```rust
pub struct RawCommit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

pub enum CommitType {
    Feat, Fix, Refactor, Docs, Test, Chore,
    Build, Ci, Perf, Style, Revert, Breaking,
    Other(String),
}

pub struct ClassifiedCommit {
    pub raw: RawCommit,
    pub commit_type: CommitType,
    pub scope: Option<String>,       // e.g. "auth" from feat(auth):
    pub description: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub breaking_description: Option<String>,
}

pub struct VersionGroup {
    pub version: String,
    pub date: String,
    pub commits: Vec<ClassifiedCommit>,
}
```

### GitParser (`git_parser.rs`)

```rust
impl GitParser {
    pub fn new(repo_path: &str) -> Self;
    pub fn parse_log(&self, depth: Option<usize>) -> Result<Vec<RawCommit>, String>;
    pub fn last_tag(&self) -> Option<String>;
    pub fn tags(&self) -> Vec<String>;
    pub fn tag_date(&self, tag: &str) -> Option<String>;
}
```

`parse_log(Some(1))` returns commits since the last tag. `parse_log(None)` returns all history.

### CommitClassifier (`commit_classifier.rs`)

```rust
impl CommitClassifier {
    pub fn new() -> Self;
    pub fn classify(&self, raw: RawCommit) -> ClassifiedCommit;
    pub fn classify_all(&self, raw_commits: Vec<RawCommit>) -> Vec<ClassifiedCommit>;
}
```

Parses `type(scope)!: description` format. Detects breaking changes from `!` suffix and `BREAKING CHANGE:` in body.

### VersionBumper (`version_bumper.rs`)

```rust
impl VersionBumper {
    pub fn new() -> Self;
    pub fn bump(&self, commits: &[ClassifiedCommit], current: Option<&str>) -> String;
    pub fn group_by_version(&self, commits: &[ClassifiedCommit], parser: &GitParser) -> Vec<VersionGroup>;
}
```

Semver logic:
- Any `breaking: true` → major bump
- Any `CommitType::Feat` → minor bump
- Everything else → patch bump

### ChangelogGenerator (`changelog_generator.rs`)

```rust
impl ChangelogGenerator {
    pub fn new() -> Self;
    pub fn generate(&self, groups: &[VersionGroup]) -> String;
}
```

Output grouped by type with emoji sections:
- ⚠ BREAKING CHANGES (first)
- ✨ Features, 🐛 Bug Fixes, ♻️ Refactor, ⚡ Performance, 📚 Documentation, 🧪 Tests, 📦 Build, 👷 CI, 🔧 Chore, 💄 Style, ⏪ Revert

### ConventionalChecker (`conventional_checker.rs`)

```rust
impl ConventionalChecker {
    pub fn new() -> Self;
    pub fn check(&self, commits: &[RawCommit]) -> Vec<CheckResult>;
}
```

Returns `CheckResult { valid, reasons }` per commit with specific diagnostics: missing colon, unknown type, subject line too long (>72 chars).

## Example Output

```
# Changelog

## [1.2.0] - 2026-06-09

### ✨ Features

- **auth**: add OAuth2 login flow (a1b2c3d)

### 🐛 Bug Fixes

- resolve null pointer in parser (e4f5g6h)

### ⚠ BREAKING CHANGES

- removed deprecated v1 API endpoint (a1b2c3d)
```

## Version Bump Examples

| Commits since last tag | Current | Next |
|------------------------|---------|------|
| `fix: crash on null` | 1.0.0 | 1.0.1 |
| `feat: add search` | 1.0.0 | 1.1.0 |
| `feat!: new API` | 1.0.0 | 2.0.0 |
| `feat(api)!: redesign` | v2.3.4 | 3.0.0 |
| (no prior tag) | — | 0.1.0 |

## Related Repos

| Repo | Role |
|------|------|
| `superinstance-architecture` | Architecture spec this tool documents |
| `beta-test-elena` | Stress-testing the 5 conservation laws |
| `negative-space-core` | Core ternary tracking crate |
