use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod commands;
mod database;
mod git;
mod markdown;
mod utils;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MarkdownStyle {
    Resume,
    Portfolio,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContributionStatusArg {
    Draft,
    Accepted,
}

impl fmt::Display for ContributionStatusArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Accepted => write!(f, "accepted"),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ConfidenceLevelArg {
    High,
    Medium,
    Low,
}

impl fmt::Display for ConfidenceLevelArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Parser)]
#[command(name = "contrack")]
#[command(version)]
#[command(about = "Turn noisy git history into structured, reusable contribution notes.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    Contribution {
        #[command(subcommand)]
        command: ContributionCommands,
    },
    Commit {
        #[command(subcommand)]
        command: CommitCommands,
    },
    #[command(visible_alias = "update")]
    Refresh {
        repo: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Generate {
        #[command(subcommand)]
        command: GenerateCommands,
    },
    Stats {
        repo: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Locations {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    Add {
        path: Option<PathBuf>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        slug: Option<String>,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Status {
        repo: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Remove {
        repo: String,
    },
}

#[derive(Subcommand)]
enum ContributionCommands {
    Add {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        overview: String,
        #[arg(long, visible_alias = "long-description")]
        description: String,
        #[arg(long, default_value = "Feature")]
        category: String,
        #[arg(long, default_value_t = 3)]
        priority: u8,
        #[arg(long = "key-commit", required = true)]
        key_commits: Vec<String>,
        #[arg(long = "related-commit")]
        related_commits: Vec<String>,
        #[arg(long = "covered-pr")]
        covered_prs: Vec<i64>,
        #[arg(long = "technical-detail")]
        technical_details: Vec<String>,
        #[arg(long = "resume-bullet")]
        resume_bullets: Vec<String>,
        #[arg(long)]
        rationale: Option<String>,
        #[arg(long, value_enum)]
        confidence: Option<ConfidenceLevelArg>,
        #[arg(long, value_enum, default_value_t = ContributionStatusArg::Draft)]
        status: ContributionStatusArg,
    },
    Edit {
        contribution: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        overview: Option<String>,
        #[arg(long, visible_alias = "long-description")]
        description: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        priority: Option<u8>,
        #[arg(long = "key-commit")]
        key_commits: Option<Vec<String>>,
        #[arg(long = "related-commit")]
        related_commits: Option<Vec<String>>,
        #[arg(long = "covered-pr")]
        covered_prs: Option<Vec<i64>>,
        #[arg(long = "technical-detail")]
        technical_details: Option<Vec<String>>,
        #[arg(long = "resume-bullet")]
        resume_bullets: Option<Vec<String>>,
        #[arg(long)]
        rationale: Option<String>,
        #[arg(long, value_enum)]
        confidence: Option<ConfidenceLevelArg>,
        #[arg(long, value_enum)]
        status: Option<ContributionStatusArg>,
        #[arg(long)]
        clear_key_commits: bool,
        #[arg(long)]
        clear_related_commits: bool,
        #[arg(long)]
        clear_covered_prs: bool,
        #[arg(long)]
        clear_technical_details: bool,
        #[arg(long)]
        clear_resume_bullets: bool,
        #[arg(long)]
        clear_rationale: bool,
        #[arg(long)]
        clear_confidence: bool,
    },
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Show {
        contribution: String,
        #[arg(long)]
        json: bool,
    },
    LinkPr {
        contribution: String,
        #[arg(required = true)]
        prs: Vec<i64>,
        #[arg(long)]
        replace: bool,
    },
    Merge {
        primary: String,
        secondary: String,
    },
}

#[derive(Subcommand)]
enum CommitCommands {
    Import {
        repo: Option<String>,
        #[arg(long)]
        all: bool,
    },
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        contribution: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    Authors {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    Markdown {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, value_enum, default_value_t = MarkdownStyle::Resume)]
        style: MarkdownStyle,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        include: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init_command(),
        Commands::Repo { command } => match command {
            RepoCommands::Add { path, name, slug } => commands::repo_add_command(path, name, slug),
            RepoCommands::List { json } => commands::repo_list_command(json),
            RepoCommands::Status { repo, json } => commands::repo_status_command(repo, json),
            RepoCommands::Remove { repo } => commands::repo_remove_command(repo),
        },
        Commands::Contribution { command } => match command {
            ContributionCommands::Add {
                repo,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                covered_prs,
                technical_details,
                resume_bullets,
                rationale,
                confidence,
                status,
            } => commands::contribution_add_command(
                repo,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                covered_prs,
                technical_details,
                resume_bullets,
                rationale,
                confidence.map(|value| value.to_string()),
                status.to_string(),
            ),
            ContributionCommands::Edit {
                contribution,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                covered_prs,
                technical_details,
                resume_bullets,
                rationale,
                confidence,
                status,
                clear_key_commits,
                clear_related_commits,
                clear_covered_prs,
                clear_technical_details,
                clear_resume_bullets,
                clear_rationale,
                clear_confidence,
            } => commands::contribution_edit_command(
                contribution,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                covered_prs,
                technical_details,
                resume_bullets,
                rationale,
                confidence.map(|value| value.to_string()),
                status.map(|value| value.to_string()),
                clear_key_commits,
                clear_related_commits,
                clear_covered_prs,
                clear_technical_details,
                clear_resume_bullets,
                clear_rationale,
                clear_confidence,
            ),
            ContributionCommands::List { repo, json } => {
                commands::contribution_list_command(repo, json)
            }
            ContributionCommands::Show { contribution, json } => {
                commands::contribution_show_command(contribution, json)
            }
            ContributionCommands::LinkPr {
                contribution,
                prs,
                replace,
            } => commands::contribution_link_pr_command(contribution, prs, replace),
            ContributionCommands::Merge { primary, secondary } => {
                commands::contribution_merge_command(primary, secondary)
            }
        },
        Commands::Commit { command } => match command {
            CommitCommands::Import { repo, all } => commands::commit_import_command(repo, all),
            CommitCommands::List {
                repo,
                contribution,
                limit,
                json,
            } => commands::commit_list_command(repo, contribution, limit, json),
            CommitCommands::Authors { repo, limit, json } => {
                commands::commit_authors_command(repo, limit, json)
            }
        },
        Commands::Refresh { repo, all } => commands::refresh_command(repo, all),
        Commands::Generate { command } => match command {
            GenerateCommands::Markdown {
                repo,
                style,
                output,
                include,
                status,
                json,
            } => commands::generate_markdown_command(repo, style, output, include, status, json),
        },
        Commands::Stats { repo, json } => commands::stats_command(repo, json),
        Commands::Locations { json } => commands::locations_command(json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_contribution_add_with_rich_metadata() {
        let cli = Cli::try_parse_from([
            "contrack",
            "contribution",
            "add",
            "--name",
            "Inventory candidate",
            "--overview",
            "Built a richer contribution record.",
            "--description",
            "Added PR coverage, confidence, rationale, and status fields.",
            "--key-commit",
            "abc1234",
            "--covered-pr",
            "42",
            "--confidence",
            "high",
            "--status",
            "accepted",
        ])
        .expect("command should parse");

        match cli.command {
            Commands::Contribution {
                command:
                    ContributionCommands::Add {
                        covered_prs,
                        confidence,
                        status,
                        ..
                    },
            } => {
                assert_eq!(covered_prs, vec![42]);
                assert!(matches!(confidence, Some(ConfidenceLevelArg::High)));
                assert!(matches!(status, ContributionStatusArg::Accepted));
            }
            _ => panic!("expected contribution add"),
        }
    }

    #[test]
    fn parse_stats_json() {
        let cli = Cli::try_parse_from(["contrack", "stats", "contrack", "--json"])
            .expect("command should parse");

        match cli.command {
            Commands::Stats { json, repo } => {
                assert!(json);
                assert_eq!(repo.as_deref(), Some("contrack"));
            }
            _ => panic!("expected stats"),
        }
    }

    #[test]
    fn parse_commit_authors_json() {
        let cli = Cli::try_parse_from([
            "contrack", "commit", "authors", "--repo", "contrack", "--limit", "5", "--json",
        ])
        .expect("command should parse");

        match cli.command {
            Commands::Commit {
                command: CommitCommands::Authors { repo, limit, json },
            } => {
                assert_eq!(repo.as_deref(), Some("contrack"));
                assert_eq!(limit, 5);
                assert!(json);
            }
            _ => panic!("expected commit authors"),
        }
    }

    #[test]
    fn parse_contribution_link_pr() {
        let cli = Cli::try_parse_from([
            "contrack",
            "contribution",
            "link-pr",
            "12",
            "441",
            "445",
            "--replace",
        ])
        .expect("command should parse");

        match cli.command {
            Commands::Contribution {
                command:
                    ContributionCommands::LinkPr {
                        contribution,
                        prs,
                        replace,
                    },
            } => {
                assert_eq!(contribution, "12");
                assert_eq!(prs, vec![441, 445]);
                assert!(replace);
            }
            _ => panic!("expected contribution link-pr"),
        }
    }
}
