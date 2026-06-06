use clap::{Parser, Subcommand};

mod git_parser;
mod commit_classifier;
mod version_bumper;
mod changelog_generator;
mod release_notes;
mod conventional_checker;

use git_parser::GitParser;
use commit_classifier::CommitClassifier;
use version_bumper::VersionBumper;
mod types;
use changelog_generator::ChangelogGenerator;
use release_notes::ReleaseNotes;
use conventional_checker::ConventionalChecker;

#[derive(Parser)]
#[command(name = "auto-changelog")]
#[command(about = "Automatic changelog generator from conventional commits")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Repository path (defaults to current directory)
    #[arg(global = true, short, long, default_value = ".")]
    repo: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate full changelog markdown
    Changelog {
        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Include all history (not just last version)
        #[arg(long)]
        all: bool,
    },
    /// Determine the next version based on commits since last tag
    NextVersion,
    /// Generate release notes for the latest version
    ReleaseNotes {
        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Check commits for conventional commit compliance
    Check {
        /// Number of recent commits to check
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let parser = GitParser::new(&cli.repo);
    let classifier = CommitClassifier::new();
    let bumper = VersionBumper::new();
    let generator = ChangelogGenerator::new();
    let release = ReleaseNotes::new();
    let checker = ConventionalChecker::new();

    match cli.command {
        Commands::Changelog { output, all } => {
            let raw_commits = match parser.parse_log(if all { None } else { Some(1) }) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing git log: {e}");
                    std::process::exit(1);
                }
            };
            let classified = classifier.classify_all(raw_commits);
            let grouped = bumper.group_by_version(&classified, &parser);
            let markdown = generator.generate(&grouped);
            if let Some(path) = output {
                std::fs::write(&path, &markdown).unwrap_or_else(|e| {
                    eprintln!("Error writing to {path}: {e}");
                    std::process::exit(1);
                });
                println!("Changelog written to {path}");
            } else {
                println!("{markdown}");
            }
        }
        Commands::NextVersion => {
            let raw_commits = match parser.parse_log(Some(1)) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing git log: {e}");
                    std::process::exit(1);
                }
            };
            let classified = classifier.classify_all(raw_commits);
            let current = bumper.current_version(&parser);
            let next = bumper.bump(&classified, current.as_deref());
            println!("{next}");
        }
        Commands::ReleaseNotes { output } => {
            let raw_commits = match parser.parse_log(Some(1)) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing git log: {e}");
                    std::process::exit(1);
                }
            };
            let classified = classifier.classify_all(raw_commits);
            let grouped = bumper.group_by_version(&classified, &parser);
            let notes = release.generate(&grouped);
            if let Some(path) = output {
                std::fs::write(&path, &notes).unwrap_or_else(|e| {
                    eprintln!("Error writing to {path}: {e}");
                    std::process::exit(1);
                });
                println!("Release notes written to {path}");
            } else {
                println!("{notes}");
            }
        }
        Commands::Check { count } => {
            let raw_commits = match parser.parse_log(Some(1)) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing git log: {e}");
                    std::process::exit(1);
                }
            };
            let to_check: Vec<_> = raw_commits.into_iter().take(count).collect();
            let results = checker.check(&to_check);
            let violations: Vec<_> = results.iter().filter(|r| !r.valid).collect();
            for result in &results {
                if result.valid {
                    println!("✅ {}", result.commit.message.lines().next().unwrap_or(""));
                } else {
                    println!("❌ {}", result.commit.message.lines().next().unwrap_or(""));
                    for reason in &result.reasons {
                        println!("   └─ {reason}");
                    }
                }
            }
            println!();
            println!("Checked {} commits, {} violations", results.len(), violations.len());
            if !violations.is_empty() {
                std::process::exit(1);
            }
        }
    }
}
