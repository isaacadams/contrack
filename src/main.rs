use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

mod commands;
mod database;
mod git;
mod markdown;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "contrack")]
#[command(about = "A CLI tool for tracking and documenting code contributions", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new contributions database
    Init {
        /// Repository URL (e.g., https://github.com/org/repo)
        #[arg(short, long)]
        repo_url: String,
        /// Organization name
        #[arg(short, long)]
        org: String,
        /// Repository name
        #[arg(short, long)]
        name: String,
        /// Repository description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Add a new contribution
    Add {
        /// Repository URL
        #[arg(short, long)]
        repo_url: String,
        /// Contribution name
        #[arg(short, long)]
        name: String,
        /// Brief overview
        #[arg(short, long)]
        overview: String,
        /// Detailed description
        #[arg(short, long)]
        description: String,
        /// Key commit hashes (comma-separated)
        #[arg(short, long)]
        key_commits: String,
        /// Related commit hashes (comma-separated, optional)
        #[arg(short, long)]
        related_commits: Option<String>,
        /// Category (Core Feature, Integration, Infrastructure, etc.)
        #[arg(short, long, default_value = "Feature")]
        category: String,
        /// Priority (1-10, higher is more important)
        #[arg(short, long, default_value_t = 5)]
        priority: u8,
    },
    /// Update commit details from git repository
    Update {
        /// Path to git repository (defaults to current directory)
        #[arg(short, long)]
        repo_path: Option<PathBuf>,
    },
    /// Generate contributions markdown file
    Generate {
        /// Repository URL
        #[arg(short, long)]
        repo_url: String,
        /// Output file path (defaults to CONTRIBUTIONS.md)
        #[arg(short, long, default_value = "CONTRIBUTIONS.md")]
        output: PathBuf,
        /// Author name to filter by (optional)
        #[arg(short, long)]
        author: Option<String>,
    },
    /// Query the database
    Query {
        #[command(subcommand)]
        subcommand: QueryCommands,
    },
    /// List repositories in the database
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    /// List all contributions for a repository
    Contributions {
        /// Repository URL
        repo_url: String,
    },
    /// Show details for a specific contribution
    Contribution {
        /// Repository URL
        repo_url: String,
        /// Contribution name
        name: String,
    },
    /// Show commits for a contribution
    Commits {
        /// Repository URL
        repo_url: String,
        /// Contribution name
        name: String,
    },
    /// Show database statistics
    Stats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            repo_url,
            org,
            name,
            description,
        } => init_command(repo_url, org, name, description),
        Commands::Add {
            repo_url,
            name,
            overview,
            description,
            key_commits,
            related_commits,
            category,
            priority,
        } => add_command(
            repo_url,
            name,
            overview,
            description,
            key_commits,
            related_commits,
            category,
            priority,
        ),
        Commands::Update { repo_path } => update_command(repo_path),
        Commands::Generate {
            repo_url,
            output,
            author,
        } => generate_command(repo_url, output, author),
        Commands::Query { subcommand } => match subcommand {
            QueryCommands::Contributions { repo_url } => query_contributions(repo_url),
            QueryCommands::Contribution { repo_url, name } => query_contribution(repo_url, name),
            QueryCommands::Commits { repo_url, name } => query_commits(repo_url, name),
            QueryCommands::Stats => query_stats(),
        },
        Commands::List { detailed } => list_repositories(detailed),
    }
}

