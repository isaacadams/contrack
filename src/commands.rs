use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::database::{Contribution, Database, Repository};
use crate::git;
use crate::markdown;

pub fn init_command(
    repo_url: String,
    org: String,
    name: String,
    description: Option<String>,
) -> Result<()> {
    use crate::config::{Config, RepositoryConfig};
    use crate::utils::get_config_path;

    let db = Database::open()?;
    let repo = Repository {
        url: repo_url.clone(),
        organization: org.clone(),
        name: name.clone(),
        description: description.clone(),
    };

    db.add_repository(&repo)?;
    
    // Auto-sync to config.toml if it exists or create it
    let config_path = get_config_path()?;
    let mut config = if config_path.exists() {
        Config::from_toml(&config_path)?
    } else {
        Config::new()
    };
    
    // Add repository to config
    config.repositories.insert(
        repo_url.clone(),
        RepositoryConfig {
            organization: org,
            name,
            description,
        },
    );
    
    // Save config
    config.to_toml(&config_path)?;
    
    println!("{} Repository initialized successfully!", "✓".green());
    println!("  URL: {}", repo.url);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_command(
    repo_url: String,
    name: String,
    overview: String,
    description: String,
    key_commits: String,
    related_commits: Option<String>,
    category: String,
    priority: u8,
) -> Result<()> {
    let db = Database::open()?;

    let key_commits_vec: Vec<String> = key_commits
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let related_commits_vec: Vec<String> = related_commits
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let contrib = Contribution {
        id: None,
        repository_url: repo_url.clone(),
        name: name.clone(),
        overview,
        description,
        key_commits: key_commits_vec,
        related_commits: related_commits_vec,
        technical_details: HashMap::new(),
        resume_bullets: Vec::new(),
        category,
        priority,
    };

    db.add_contribution(&contrib)?;
    println!("{} Contribution '{}' added successfully!", "✓".green(), name);
    Ok(())
}

pub fn update_command(repo_path: Option<PathBuf>) -> Result<()> {
    let db = Database::open()?;
    let repo_path = repo_path.unwrap_or_else(|| PathBuf::from("."));

    println!("Extracting commit details from git repository...");
    let commits = git::extract_commits_from_repo(&repo_path)?;

    println!("Found {} commits to process", commits.len());

    // Get all contributions to match commits
    let repos = db.get_all_repositories()?;
    let mut processed = 0;

    for commit in &commits {
        // Try to find matching contribution by checking if commit hash is in key_commits or related_commits
        let mut contrib_id = None;
        for repo in &repos {
            if repo.url != commit.repository_url {
                continue;
            }
            let contribs = db.get_contributions(&repo.url)?;
            for contrib in contribs {
                if contrib.key_commits.iter().any(|c| commit.hash.starts_with(c)) ||
                   contrib.related_commits.iter().any(|c| commit.hash.starts_with(c)) {
                    if let Some(id) = contrib.id {
                        contrib_id = Some(id);
                        break;
                    }
                }
            }
            if contrib_id.is_some() {
                break;
            }
        }
        
        let mut commit_with_id = commit.clone();
        commit_with_id.contribution_id = contrib_id;

        db.add_commit(&commit_with_id)?;
        processed += 1;

        if processed % 10 == 0 {
            println!("Processed {} commits...", processed);
        }
    }

    println!("{} Update complete: {} processed", 
             "✓".green(), processed);
    Ok(())
}

pub fn generate_command(
    repo_url: String,
    output: PathBuf,
    author: Option<String>,
) -> Result<()> {
    let db = Database::open()?;
    let contributions = db.get_contributions(&repo_url)?;

    if contributions.is_empty() {
        println!("{} No contributions found for repository: {}", 
                 "⚠".yellow(), repo_url);
        return Ok(());
    }

    // Get commits for each contribution
    let mut contributions_with_commits = Vec::new();
    for contrib in &contributions {
        let commits = db.get_commits_for_contribution(&repo_url, &contrib.name)?;
        contributions_with_commits.push((contrib.clone(), commits));
    }

    let markdown = markdown::generate_markdown(
        &repo_url,
        &contributions_with_commits,
        author.as_deref(),
    )?;

    std::fs::write(&output, markdown)
        .with_context(|| format!("Failed to write to {:?}", output))?;

    println!("{} Generated contributions markdown: {:?}", 
             "✓".green(), output);
    println!("  {} contributions documented", contributions.len());
    Ok(())
}

pub fn query_contributions(repo_url: String) -> Result<()> {
    let db = Database::open()?;
    let contributions = db.get_contributions(&repo_url)?;

    if contributions.is_empty() {
        println!("No contributions found for repository: {}", repo_url);
        return Ok(());
    }

    println!("\n{} Contributions for {}", "📋".blue(), repo_url);
    println!("{}", "=".repeat(80));

    for contrib in contributions {
        println!("\n{} {}", "•".green(), contrib.name.bold());
        println!("  Category: {} | Priority: {}", contrib.category, contrib.priority);
        println!("  Overview: {}", contrib.overview);
        println!("  Key Commits: {}", contrib.key_commits.len());
    }

    Ok(())
}

pub fn query_contribution(repo_url: String, name: String) -> Result<()> {
    let db = Database::open()?;
    let contrib = db.get_contribution(&repo_url, &name)?
        .with_context(|| format!("Contribution '{}' not found", name))?;

    println!("\n{} Contribution: {}", "📄".blue(), contrib.name.bold());
    println!("{}", "=".repeat(80));
    println!("Repository: {}", contrib.repository_url);
    println!("Category: {} | Priority: {}", contrib.category, contrib.priority);
    println!("\nOverview:\n{}", contrib.overview);
    println!("\nDescription:\n{}", contrib.description);

    if !contrib.key_commits.is_empty() {
        println!("\nKey Commits ({}):", contrib.key_commits.len());
        for commit in &contrib.key_commits {
            println!("  - {}", commit);
        }
    }

    if !contrib.related_commits.is_empty() {
        println!("\nRelated Commits ({}):", contrib.related_commits.len());
        for commit in &contrib.related_commits[..contrib.related_commits.len().min(5)] {
            println!("  - {}", commit);
        }
        if contrib.related_commits.len() > 5 {
            println!("  ... and {} more", contrib.related_commits.len() - 5);
        }
    }

    if !contrib.technical_details.is_empty() {
        println!("\nTechnical Details:");
        for (key, value) in &contrib.technical_details {
            println!("  {}: {}", key, value);
        }
    }

    if !contrib.resume_bullets.is_empty() {
        println!("\nResume Bullets ({}):", contrib.resume_bullets.len());
        for (i, bullet) in contrib.resume_bullets.iter().enumerate() {
            println!("  {}. {}", i + 1, bullet);
        }
    }

    Ok(())
}

pub fn query_commits(repo_url: String, name: String) -> Result<()> {
    let db = Database::open()?;
    let commits = db.get_commits_for_contribution(&repo_url, &name)?;

    if commits.is_empty() {
        println!("No commits found for contribution '{}'", name);
        return Ok(());
    }

    println!("\n{} Commits for '{}'", "🔍".blue(), name.bold());
    println!("{}", "=".repeat(80));

    for commit in commits {
        println!("\n{} {}", "•".green(), commit.hash[..8].yellow());
        println!("  Author: {} <{}>", commit.author, commit.author_email);
        println!("  Date: {}", commit.date);
        println!("  Message: {}", commit.message);
        if let (Some(added), Some(deleted)) = (commit.lines_added, commit.lines_deleted) {
            println!("  Changes: +{} -{}", added.to_string().green(), deleted.to_string().red());
        }
    }

    Ok(())
}

pub fn query_stats() -> Result<()> {
    let db = Database::open()?;
    let stats = db.get_statistics()?;

    println!("\n{} Database Statistics", "📊".blue());
    println!("{}", "=".repeat(80));
    println!("Repositories: {}", stats.get("repositories").unwrap_or(&0));
    println!("Contributions: {}", stats.get("contributions").unwrap_or(&0));
    println!("Commits: {}", stats.get("commits").unwrap_or(&0));
    println!("Agent Rules: {}", stats.get("agent_rules").unwrap_or(&0));
    println!("Prompts: {}", stats.get("prompts").unwrap_or(&0));

    Ok(())
}

pub fn list_repositories(detailed: bool) -> Result<()> {
    let db = Database::open()?;
    let repos = db.get_all_repositories()?;

    if repos.is_empty() {
        println!("No repositories found in database");
        return Ok(());
    }

    println!("\n{} Repositories", "📦".blue());
    println!("{}", "=".repeat(80));

    for repo in repos {
        println!("\n{} {}", "•".green(), repo.name.bold());
        println!("  URL: {}", repo.url);
        println!("  Organization: {}", repo.organization);
        if let Some(desc) = repo.description {
            println!("  Description: {}", desc);
        }

        if detailed {
            let contribs = db.get_contributions(&repo.url)?;
            println!("  Contributions: {}", contribs.len());
        }
    }

    Ok(())
}

pub fn locations_command() -> Result<()> {
    use crate::utils::{get_contrack_dir, get_database_path};
    use directories::ProjectDirs;

    println!("\n{} Contrack Database Locations", "📍".blue());
    println!("{}", "=".repeat(80));

    // Get current database path (this will be the active one)
    let current_db_path = get_database_path()?;
    let is_project_local = get_contrack_dir().is_some();

    // Display current location
    println!("\n{} Current Database (Active)", "•".green());
    if is_project_local {
        println!("  Type: {}", "Project-Local".bold().green());
        if let Some(contrack_dir) = get_contrack_dir() {
            println!("  Directory: {}", contrack_dir.display());
        }
    } else {
        println!("  Type: {}", "Global".bold().yellow());
        let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
            .context("Failed to determine application data directory")?;
        println!("  Directory: {}", project_dirs.data_dir().display());
    }
    println!("  Database: {}", current_db_path.display());
    println!("  Exists: {}", if current_db_path.exists() { "Yes".green() } else { "No".red() });

    // Show project-local location if different from current
    if let Some(contrack_dir) = get_contrack_dir() {
        let project_db = contrack_dir.join("contributions.db");
        if project_db != current_db_path {
            println!("\n{} Project-Local Location", "•".blue());
            println!("  Type: {}", "Project-Local".bold().green());
            println!("  Directory: {}", contrack_dir.display());
            println!("  Database: {}", project_db.display());
            println!("  Exists: {}", if project_db.exists() { "Yes".green() } else { "No".red() });
        }
    }

    // Show global location
    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine application data directory")?;
    let global_db = project_dirs.data_dir().join("contributions.db");
    
    if global_db != current_db_path {
        println!("\n{} Global Location", "•".blue());
        println!("  Type: {}", "Global".bold().yellow());
        println!("  Directory: {}", project_dirs.data_dir().display());
        println!("  Database: {}", global_db.display());
        println!("  Exists: {}", if global_db.exists() { "Yes".green() } else { "No".red() });
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod locations_tests {
    use super::*;

    #[test]
    fn test_locations_command() {
        // Test that the command doesn't panic and returns Ok
        let result = locations_command();
        assert!(result.is_ok());
    }
}

pub fn config_sync_command() -> Result<()> {
    use crate::utils::get_config_path;

    let db = Database::open()?;
    let config = db.load_config_from_db()?;
    let config_path = get_config_path()?;
    
    config.to_toml(&config_path)?;
    println!("{} Configuration synced to: {}", "✓".green(), config_path.display());
    Ok(())
}

pub fn config_load_command() -> Result<()> {
    use crate::config::Config;
    use crate::utils::get_config_path;

    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Config file not found: {:?}", config_path));
    }
    
    let config = Config::from_toml(&config_path)?;
    let db = Database::open()?;
    db.load_config_to_db(&config)?;
    
    println!("{} Configuration loaded from: {}", "✓".green(), config_path.display());
    Ok(())
}

pub fn config_add_org_command(id: String, name: String, description: Option<String>) -> Result<()> {
    use crate::config::{Config, Organization};
    use crate::utils::get_config_path;

    let config_path = get_config_path()?;
    let mut config = if config_path.exists() {
        Config::from_toml(&config_path)?
    } else {
        Config::new()
    };
    
    // Add organization to config
    config.organizations.insert(
        id.clone(),
        Organization {
            name,
            description,
        },
    );
    
    // Save config
    config.to_toml(&config_path)?;
    
    // Also update database
    let db = Database::open()?;
    db.load_config_to_db(&config)?;
    
    println!("{} Organization '{}' added", "✓".green(), id);
    Ok(())
}

pub fn config_add_repo_command(url: String, org: String, name: String, description: Option<String>) -> Result<()> {
    use crate::config::{Config, RepositoryConfig};
    use crate::database::Repository;
    use crate::utils::get_config_path;

    let config_path = get_config_path()?;
    let mut config = if config_path.exists() {
        Config::from_toml(&config_path)?
    } else {
        Config::new()
    };
    
    // Add repository to config
    config.repositories.insert(
        url.clone(),
        RepositoryConfig {
            organization: org.clone(),
            name: name.clone(),
            description: description.clone(),
        },
    );
    
    // Save config
    config.to_toml(&config_path)?;
    
    // Also update database
    let db = Database::open()?;
    let repo = Repository {
        url,
        organization: org,
        name,
        description,
    };
    db.add_repository(&repo)?;
    
    println!("{} Repository added", "✓".green());
    Ok(())
}

pub fn loadout_list_command() -> Result<()> {
    let db = Database::open()?;
    let loadouts = db.list_loadouts()?;

    if loadouts.is_empty() {
        println!("No loadouts found");
        return Ok(());
    }

    println!("\n{} Loadouts", "📦".blue());
    println!("{}", "=".repeat(80));

    for (id, name, is_default) in loadouts {
        let default_marker = if is_default { " (default)" } else { "" };
        println!("\n{} {} [ID: {}]{}", "•".green(), name.bold(), id, default_marker);
    }

    println!();
    Ok(())
}

pub fn loadout_create_command(name: String) -> Result<()> {
    let db = Database::open()?;
    
    // Check if loadout already exists
    if db.get_loadout_id(&name)?.is_some() {
        return Err(anyhow::anyhow!("Loadout '{}' already exists", name));
    }
    
    db.create_loadout(&name)?;
    println!("{} Loadout '{}' created", "✓".green(), name);
    Ok(())
}

pub fn loadout_load_command(name: String) -> Result<()> {
    let db = Database::open()?;
    db.load_loadout(&name)?;
    println!("{} Loadout '{}' loaded", "✓".green(), name);
    Ok(())
}

pub fn loadout_save_command(name: String) -> Result<()> {
    let db = Database::open()?;
    
    // Create loadout if it doesn't exist
    if db.get_loadout_id(&name)?.is_none() {
        db.create_loadout(&name)?;
    }
    
    db.save_current_to_loadout(&name)?;
    println!("{} Current prompts and rules saved to loadout '{}'", "✓".green(), name);
    Ok(())
}

pub fn loadout_delete_command(name: String) -> Result<()> {
    let db = Database::open()?;
    db.delete_loadout(&name)?;
    println!("{} Loadout '{}' deleted", "✓".green(), name);
    Ok(())
}

pub fn loadout_reload_default_command() -> Result<()> {
    let db = Database::open()?;
    db.reload_default_loadout()?;
    println!("{} Default loadout reloaded", "✓".green());
    Ok(())
}

pub fn ai_command() -> Result<()> {
    let db = Database::open()?;
    
    // Introduction
    println!("Contrack - Contribution Tracking Tool for AI Agents");
    println!("==================================================\n");
    println!("Contrack is a CLI tool designed to help AI agents track and document code contributions across repositories.");
    println!("It maintains a SQLite database of repositories, contributions, commits, agent rules, and prompts.\n");
    
    // Instructions
    println!("HOW TO USE THIS TOOL:");
    println!("---------------------");
    println!("1. Review the agent rules below to understand how to work with the contributions database");
    println!("2. Review the available prompts to see what tasks you can help with");
    println!("3. Ask the user which prompt they would like to execute");
    println!("4. Use the contrack CLI commands to interact with the database");
    println!("5. Always maintain consistency with existing data patterns\n");
    
    // Commands list
    println!("AVAILABLE COMMANDS:");
    println!("-------------------");
    println!("  contrack init          - Initialize a new repository");
    println!("  contrack add           - Add a new contribution");
    println!("  contrack update        - Update commit details from git");
    println!("  contrack generate      - Generate contributions markdown file");
    println!("  contrack query         - Query the database (contributions, commits, stats)");
    println!("  contrack list          - List repositories");
    println!("  contrack locations     - List all database locations");
    println!("  contrack config        - Manage configuration file");
    println!("  contrack loadout       - Manage prompt and rule loadouts");
    println!("  contrack ai            - Show this AI configuration prompt\n");
    
    // Agent rules
    println!("AGENT RULES:");
    println!("------------");
    let rules = db.get_all_agent_rules()?;
    if rules.is_empty() {
        println!("No agent rules found in database.\n");
    } else {
        for (i, (name, instruction, priority, category)) in rules.iter().enumerate() {
            println!("\n{}. {} [Priority: {}, Category: {}]", 
                i + 1, 
                name, 
                priority,
                category.as_deref().unwrap_or("Uncategorized")
            );
            println!("   {}", instruction.replace('\n', "\n   "));
        }
        println!();
    }
    
    // Available prompts
    println!("AVAILABLE PROMPTS:");
    println!("------------------");
    let prompts = db.get_all_prompts()?;
    if prompts.is_empty() {
        println!("No prompts found in database.\n");
    } else {
        for (i, (name, prompt_text, description, category)) in prompts.iter().enumerate() {
            println!("\n{}. {} [Category: {}]", 
                i + 1, 
                name,
                category.as_deref().unwrap_or("Uncategorized")
            );
            if let Some(desc) = description {
                println!("   Description: {}", desc);
            }
            println!("   Prompt: {}", prompt_text.replace('\n', "\n   "));
        }
        println!();
    }
    
    // Final instruction
    println!("NEXT STEPS:");
    println!("-----------");
    println!("Please ask the user which prompt they would like to execute from the list above.");
    println!("Once they select a prompt, you can help them execute it using the contrack tool.");
    
    Ok(())
}

#[cfg(test)]
mod ai_tests {
    use super::*;

    #[test]
    fn test_ai_command() {
        // Test that the command doesn't panic and returns Ok
        let result = ai_command();
        assert!(result.is_ok());
    }
}

