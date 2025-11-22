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
    let db = Database::open()?;
    let repo = Repository {
        url: repo_url.clone(),
        organization: org,
        name,
        description,
    };

    db.add_repository(&repo)?;
    println!("{} Repository initialized successfully!", "✓".green());
    println!("  URL: {}", repo.url);
    Ok(())
}

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

