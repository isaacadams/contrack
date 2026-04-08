use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
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
    /// Create a local .contrack workspace in the current project.
    Init,
    /// Track repositories that should feed contribution data.
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// Create, update, and inspect contributions.
    Contribution {
        #[command(subcommand)]
        command: ContributionCommands,
    },
    /// Import and inspect git commit evidence.
    Commit {
        #[command(subcommand)]
        command: CommitCommands,
    },
    /// Refresh imported commit metadata for tracked repositories.
    #[command(visible_alias = "update")]
    Refresh {
        /// Refresh a single tracked repository by slug, name, path, or remote URL.
        repo: Option<String>,
        /// Refresh every tracked repository.
        #[arg(long)]
        all: bool,
    },
    /// Generate polished Markdown output.
    Generate {
        #[command(subcommand)]
        command: GenerateCommands,
    },
    /// Show database and repository statistics.
    Stats {
        /// Limit stats to one tracked repository.
        repo: Option<String>,
    },
    /// Show active and fallback database locations.
    Locations,
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Track a git repository.
    Add {
        /// Repository path. Defaults to the current directory.
        path: Option<PathBuf>,
        /// Friendly display name. Defaults to the repository directory name.
        #[arg(long)]
        name: Option<String>,
        /// Stable short identifier used in commands.
        #[arg(long)]
        slug: Option<String>,
    },
    /// List tracked repositories.
    List,
    /// Remove a tracked repository and all of its stored data.
    Remove {
        /// Repository slug, name, path, or remote URL.
        repo: String,
    },
}

#[derive(Subcommand)]
enum ContributionCommands {
    /// Add a contribution.
    Add {
        /// Repository slug, name, path, or remote URL. Defaults to the current repo.
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
        /// Repeat for each key commit hash or short hash.
        #[arg(long = "key-commit", required = true)]
        key_commits: Vec<String>,
        /// Repeat for each related commit hash or short hash.
        #[arg(long = "related-commit")]
        related_commits: Vec<String>,
        /// Optional implementation details to include in generated Markdown.
        #[arg(long = "technical-detail")]
        technical_details: Vec<String>,
        /// Optional resume-ready bullet points.
        #[arg(long = "resume-bullet")]
        resume_bullets: Vec<String>,
    },
    /// Edit a contribution by id or name.
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
        #[arg(long = "technical-detail")]
        technical_details: Option<Vec<String>>,
        #[arg(long = "resume-bullet")]
        resume_bullets: Option<Vec<String>>,
        #[arg(long)]
        clear_key_commits: bool,
        #[arg(long)]
        clear_related_commits: bool,
        #[arg(long)]
        clear_technical_details: bool,
        #[arg(long)]
        clear_resume_bullets: bool,
    },
    /// List contributions.
    List {
        /// Repository slug, name, path, or remote URL. Defaults to the current repo.
        #[arg(long)]
        repo: Option<String>,
    },
    /// Show one contribution in detail.
    Show { contribution: String },
}

#[derive(Subcommand)]
enum CommitCommands {
    /// Import commit metadata for tracked repositories.
    Import {
        /// Repository slug, name, path, or remote URL.
        repo: Option<String>,
        /// Import every tracked repository.
        #[arg(long)]
        all: bool,
    },
    /// List imported commits.
    List {
        /// Repository slug, name, path, or remote URL. Defaults to the current repo.
        #[arg(long)]
        repo: Option<String>,
        /// Filter to commits linked by hash to a contribution.
        #[arg(long)]
        contribution: Option<String>,
        /// Maximum number of commits to show.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate structured Markdown from stored contributions.
    Markdown {
        /// Repository slug, name, path, or remote URL. Defaults to the current repo.
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, value_enum, default_value_t = MarkdownStyle::Resume)]
        style: MarkdownStyle,
        /// Write Markdown to a file instead of stdout.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init_command(),
        Commands::Repo { command } => match command {
            RepoCommands::Add { path, name, slug } => commands::repo_add_command(path, name, slug),
            RepoCommands::List => commands::repo_list_command(),
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
                technical_details,
                resume_bullets,
            } => commands::contribution_add_command(
                repo,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                technical_details,
                resume_bullets,
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
                technical_details,
                resume_bullets,
                clear_key_commits,
                clear_related_commits,
                clear_technical_details,
                clear_resume_bullets,
            } => commands::contribution_edit_command(
                contribution,
                name,
                overview,
                description,
                category,
                priority,
                key_commits,
                related_commits,
                technical_details,
                resume_bullets,
                clear_key_commits,
                clear_related_commits,
                clear_technical_details,
                clear_resume_bullets,
            ),
            ContributionCommands::List { repo } => commands::contribution_list_command(repo),
            ContributionCommands::Show { contribution } => {
                commands::contribution_show_command(contribution)
            }
        },
        Commands::Commit { command } => match command {
            CommitCommands::Import { repo, all } => commands::commit_import_command(repo, all),
            CommitCommands::List {
                repo,
                contribution,
                limit,
            } => commands::commit_list_command(repo, contribution, limit),
        },
        Commands::Refresh { repo, all } => commands::refresh_command(repo, all),
        Commands::Generate { command } => match command {
            GenerateCommands::Markdown {
                repo,
                style,
                output,
            } => commands::generate_markdown_command(repo, style, output),
        },
        Commands::Stats { repo } => commands::stats_command(repo),
        Commands::Locations => commands::locations_command(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_contribution_add() {
        let cli = Cli::try_parse_from([
            "contrack",
            "contribution",
            "add",
            "--name",
            "CLI overhaul",
            "--overview",
            "Simplified the command surface.",
            "--description",
            "Rebuilt the CLI around repositories, contributions, commits, and Markdown output.",
            "--key-commit",
            "abc1234",
            "--category",
            "Tooling",
            "--priority",
            "5",
        ])
        .expect("command should parse");

        match cli.command {
            Commands::Contribution {
                command:
                    ContributionCommands::Add {
                        name,
                        key_commits,
                        priority,
                        ..
                    },
            } => {
                assert_eq!(name, "CLI overhaul");
                assert_eq!(key_commits, vec!["abc1234"]);
                assert_eq!(priority, 5);
            }
            _ => panic!("expected contribution add"),
        }
    }
}
