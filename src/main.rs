use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

mod commands;
mod config;
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
        #[arg(long)]
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
    /// List all known contrack database locations
    Locations,
    /// Manage configuration file
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },
    /// Manage prompt and rule loadouts
    Loadout {
        #[command(subcommand)]
        subcommand: LoadoutCommands,
    },
    /// Output AI agent configuration prompt
    Ai,
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

#[derive(Subcommand)]
enum ConfigCommands {
    /// Sync database to config.toml (write current state to file)
    Sync,
    /// Load config.toml into database (read file and update database)
    Load,
    /// Add a new organization
    AddOrg {
        /// Organization identifier (key in config)
        #[arg(short, long)]
        id: String,
        /// Organization name
        #[arg(short, long)]
        name: String,
        /// Organization description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Add a new repository
    AddRepo {
        /// Repository URL
        #[arg(short, long)]
        url: String,
        /// Organization identifier
        #[arg(short, long)]
        org: String,
        /// Repository name
        #[arg(short, long)]
        name: String,
        /// Repository description
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum LoadoutCommands {
    /// List all loadouts
    List,
    /// Create a new empty loadout
    Create {
        /// Loadout name
        name: String,
    },
    /// Load a loadout (replace current prompts/rules)
    Load {
        /// Loadout name
        name: String,
    },
    /// Save current prompts/rules to a loadout
    Save {
        /// Loadout name
        name: String,
    },
    /// Delete a loadout
    Delete {
        /// Loadout name
        name: String,
    },
    /// Reload the default loadout
    ReloadDefault,
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
        Commands::Locations => locations_command(),
        Commands::Config { subcommand } => match subcommand {
            ConfigCommands::Sync => config_sync_command(),
            ConfigCommands::Load => config_load_command(),
            ConfigCommands::AddOrg { id, name, description } => config_add_org_command(id, name, description),
            ConfigCommands::AddRepo { url, org, name, description } => config_add_repo_command(url, org, name, description),
        },
        Commands::Loadout { subcommand } => match subcommand {
            LoadoutCommands::List => loadout_list_command(),
            LoadoutCommands::Create { name } => loadout_create_command(name),
            LoadoutCommands::Load { name } => loadout_load_command(name),
            LoadoutCommands::Save { name } => loadout_save_command(name),
            LoadoutCommands::Delete { name } => loadout_delete_command(name),
            LoadoutCommands::ReloadDefault => loadout_reload_default_command(),
        },
        Commands::Ai => ai_command(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command_parsing() {
        // Test that add command can parse with both repo_url and related_commits
        // This ensures there's no short option conflict
        let args = vec![
            "contrack",
            "add",
            "--repo-url", "https://github.com/test/repo",
            "--name", "Test Feature",
            "--overview", "Test overview",
            "--description", "Test description",
            "--key-commits", "abc123",
            "--related-commits", "def456",
            "--category", "Feature",
            "--priority", "5",
        ];
        
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Add {
                repo_url,
                name,
                overview,
                description,
                key_commits,
                related_commits,
                category,
                priority,
            } => {
                assert_eq!(repo_url, "https://github.com/test/repo");
                assert_eq!(name, "Test Feature");
                assert_eq!(overview, "Test overview");
                assert_eq!(description, "Test description");
                assert_eq!(key_commits, "abc123");
                assert_eq!(related_commits, Some("def456".to_string()));
                assert_eq!(category, "Feature");
                assert_eq!(priority, 5);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_with_short_options() {
        // Test that short option -r works for repo_url
        let args = vec![
            "contrack",
            "add",
            "-r", "https://github.com/test/repo",
            "-n", "Test Feature",
            "-o", "Test overview",
            "-d", "Test description",
            "-k", "abc123",
            "--related-commits", "def456", // Should use long form
            "-c", "Feature",
            "-p", "5",
        ];
        
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Add { repo_url, .. } => {
                assert_eq!(repo_url, "https://github.com/test/repo");
            }
            _ => panic!("Expected Add command"),
        }
    }
}

