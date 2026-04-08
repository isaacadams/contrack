use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::database::{
    Contribution, Database, DatabaseStats, NewContribution, NewRepository, StoredCommit,
    TrackedRepository,
};
use crate::git;
use crate::markdown::{self, ContributionEvidence};
use crate::utils::{
    canonicalize_path, get_contrack_dir, get_database_path, get_global_database_path,
    initialize_local_workspace, shorten_hash, slugify,
};
use crate::MarkdownStyle;

pub fn init_command() -> Result<()> {
    let workspace = initialize_local_workspace()?;
    let database_path = workspace.join(crate::utils::DATABASE_FILE_NAME);
    Database::open_at(&database_path)?;

    println!("Initialized contrack in {}", workspace.display());
    println!("Database: {}", database_path.display());
    println!("Next: contrack repo add .");
    Ok(())
}

pub fn repo_add_command(
    path: Option<PathBuf>,
    name: Option<String>,
    slug: Option<String>,
) -> Result<()> {
    let db = Database::open()?;
    let requested_path = path.unwrap_or_else(|| PathBuf::from("."));
    let metadata = git::inspect_repository(&requested_path)?;
    let repository_name = name.unwrap_or(metadata.inferred_name);
    let repository_slug = slug.unwrap_or_else(|| slugify(&repository_name));

    if repository_slug.is_empty() {
        return Err(anyhow!(
            "Could not derive a usable repository slug. Pass `--slug`."
        ));
    }

    let repository = db.upsert_repository(&NewRepository {
        slug: repository_slug,
        name: repository_name,
        local_path: metadata.root_path.display().to_string(),
        remote_url: metadata.normalized_remote_url,
    })?;

    println!("Tracked repository `{}`", repository.slug);
    println!("Path: {}", repository.local_path);
    if let Some(remote_url) = repository.remote_url {
        println!("Remote: {}", remote_url);
    }
    Ok(())
}

pub fn repo_list_command() -> Result<()> {
    let db = Database::open()?;
    let repositories = db.list_repositories()?;

    if repositories.is_empty() {
        println!("No tracked repositories yet.");
        println!("Start with: contrack repo add .");
        return Ok(());
    }

    for repository in repositories {
        println!("{}  {}", repository.slug, repository.name);
        println!("  path: {}", repository.local_path);
        if let Some(remote_url) = repository.remote_url {
            println!("  remote: {}", remote_url);
        }
    }

    Ok(())
}

pub fn repo_remove_command(selector: String) -> Result<()> {
    let db = Database::open()?;
    let repository = db
        .get_repository(&selector)?
        .with_context(|| format!("No tracked repository matched '{selector}'."))?;
    db.remove_repository(repository.id)?;

    println!(
        "Removed repository `{}` and its stored contributions and commits.",
        repository.slug
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn contribution_add_command(
    repo_selector: Option<String>,
    name: String,
    overview: String,
    description: String,
    category: String,
    priority: u8,
    key_commits: Vec<String>,
    related_commits: Vec<String>,
    technical_details: Vec<String>,
    resume_bullets: Vec<String>,
) -> Result<()> {
    validate_priority(priority)?;
    let db = Database::open()?;
    let repository = resolve_repository(&db, repo_selector.as_deref())?;

    let contribution = db.add_contribution(&NewContribution {
        repository_id: repository.id,
        name: name.clone(),
        overview,
        description,
        category,
        priority,
        key_commit_refs: sanitize_refs(key_commits),
        related_commit_refs: sanitize_refs(related_commits),
        technical_details: sanitize_lines(technical_details),
        resume_bullets: sanitize_lines(resume_bullets),
    })?;

    println!(
        "Saved contribution {} for repository `{}`.",
        contribution.id, repository.slug
    );
    println!("Name: {}", name);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn contribution_edit_command(
    selector: String,
    name: Option<String>,
    overview: Option<String>,
    description: Option<String>,
    category: Option<String>,
    priority: Option<u8>,
    key_commits: Option<Vec<String>>,
    related_commits: Option<Vec<String>>,
    technical_details: Option<Vec<String>>,
    resume_bullets: Option<Vec<String>>,
    clear_key_commits: bool,
    clear_related_commits: bool,
    clear_technical_details: bool,
    clear_resume_bullets: bool,
) -> Result<()> {
    let db = Database::open()?;
    let mut contribution = db
        .get_contribution(&selector)?
        .with_context(|| format!("No contribution matched '{selector}'."))?;

    if let Some(priority) = priority {
        validate_priority(priority)?;
        contribution.priority = priority;
    }
    if let Some(name) = name {
        contribution.name = name;
    }
    if let Some(overview) = overview {
        contribution.overview = overview;
    }
    if let Some(description) = description {
        contribution.description = description;
    }
    if let Some(category) = category {
        contribution.category = category;
    }
    if clear_key_commits {
        contribution.key_commit_refs.clear();
    } else if let Some(key_commits) = key_commits {
        contribution.key_commit_refs = sanitize_refs(key_commits);
    }
    if clear_related_commits {
        contribution.related_commit_refs.clear();
    } else if let Some(related_commits) = related_commits {
        contribution.related_commit_refs = sanitize_refs(related_commits);
    }
    if clear_technical_details {
        contribution.technical_details.clear();
    } else if let Some(technical_details) = technical_details {
        contribution.technical_details = sanitize_lines(technical_details);
    }
    if clear_resume_bullets {
        contribution.resume_bullets.clear();
    } else if let Some(resume_bullets) = resume_bullets {
        contribution.resume_bullets = sanitize_lines(resume_bullets);
    }

    db.update_contribution(&contribution)?;
    println!(
        "Updated contribution {} ({})",
        contribution.id, contribution.name
    );
    Ok(())
}

pub fn contribution_list_command(repo_selector: Option<String>) -> Result<()> {
    let db = Database::open()?;
    let repository_id = match repo_selector.as_deref() {
        Some(_) | None if should_infer_repo(&db, repo_selector.as_deref())? => {
            Some(resolve_repository(&db, repo_selector.as_deref())?.id)
        }
        _ => None,
    };

    let contributions = db.list_contributions(repository_id)?;
    if contributions.is_empty() {
        println!("No contributions found.");
        return Ok(());
    }

    for contribution in contributions {
        println!(
            "{}  [{}] p{}  {}",
            contribution.id, contribution.repository_slug, contribution.priority, contribution.name
        );
        println!("  {}", contribution.overview);
    }

    Ok(())
}

pub fn contribution_show_command(selector: String) -> Result<()> {
    let db = Database::open()?;
    let contribution = db
        .get_contribution(&selector)?
        .with_context(|| format!("No contribution matched '{selector}'."))?;
    let commits = db.list_all_commits_for_repository(contribution.repository_id)?;
    let evidence = build_evidence(&contribution, &commits);

    println!("{} ({})", contribution.name, contribution.id);
    println!("Repository: {}", contribution.repository_slug);
    println!("Category: {}", contribution.category);
    println!("Priority: {}", contribution.priority);
    println!();
    println!("{}", contribution.overview);
    println!();
    println!("{}", contribution.description);

    if !contribution.technical_details.is_empty() {
        println!();
        println!("Technical details:");
        for detail in &contribution.technical_details {
            println!("- {}", detail);
        }
    }

    if !contribution.resume_bullets.is_empty() {
        println!();
        println!("Resume bullets:");
        for bullet in &contribution.resume_bullets {
            println!("- {}", bullet);
        }
    }

    print_commit_matches(
        "Key commits",
        &evidence.key_commits,
        &evidence.unresolved_key_refs,
    );
    print_commit_matches(
        "Related commits",
        &evidence.related_commits,
        &evidence.unresolved_related_refs,
    );

    Ok(())
}

pub fn commit_import_command(repo_selector: Option<String>, all: bool) -> Result<()> {
    refresh_inner(repo_selector, all)
}

pub fn refresh_command(repo_selector: Option<String>, all: bool) -> Result<()> {
    refresh_inner(repo_selector, all)
}

pub fn commit_list_command(
    repo_selector: Option<String>,
    contribution_selector: Option<String>,
    limit: usize,
) -> Result<()> {
    let db = Database::open()?;

    if let Some(contribution_selector) = contribution_selector {
        let contribution = db
            .get_contribution(&contribution_selector)?
            .with_context(|| format!("No contribution matched '{contribution_selector}'."))?;
        let commits = db.list_all_commits_for_repository(contribution.repository_id)?;
        let evidence = build_evidence(&contribution, &commits);
        let mut matched = evidence.key_commits;
        matched.extend(evidence.related_commits);
        matched.truncate(limit);

        if matched.is_empty() {
            println!(
                "No imported commits matched contribution '{}'.",
                contribution.name
            );
            return Ok(());
        }

        for commit in matched {
            print_commit_line(&commit);
        }
        return Ok(());
    }

    let repository = resolve_repository(&db, repo_selector.as_deref())?;
    let commits = db.list_commits(Some(repository.id), limit)?;
    if commits.is_empty() {
        println!("No imported commits for repository `{}`.", repository.slug);
        return Ok(());
    }

    for commit in commits {
        print_commit_line(&commit);
    }

    Ok(())
}

pub fn generate_markdown_command(
    repo_selector: Option<String>,
    style: MarkdownStyle,
    output: Option<PathBuf>,
) -> Result<()> {
    let db = Database::open()?;
    let repository = resolve_repository(&db, repo_selector.as_deref())?;
    let contributions = db.list_contributions(Some(repository.id))?;

    if contributions.is_empty() {
        return Err(anyhow!(
            "No contributions found for repository `{}`. Add one with `contrack contribution add`.",
            repository.slug
        ));
    }

    let commits = db.list_all_commits_for_repository(repository.id)?;
    let items = contributions
        .iter()
        .map(|contribution| build_evidence(contribution, &commits))
        .collect::<Vec<_>>();
    let rendered = markdown::render_markdown(&repository, &items, style);

    if let Some(output) = output {
        fs::write(&output, rendered)
            .with_context(|| format!("Failed to write {}", output.display()))?;
        println!("Wrote Markdown to {}", output.display());
    } else {
        print!("{}", rendered);
    }

    Ok(())
}

pub fn stats_command(repo_selector: Option<String>) -> Result<()> {
    let db = Database::open()?;
    let (label, stats) = if let Some(selector) = repo_selector {
        let repository = db
            .get_repository(&selector)?
            .with_context(|| format!("No tracked repository matched '{selector}'."))?;
        (
            format!("Repository `{}`", repository.slug),
            db.stats(Some(repository.id))?,
        )
    } else {
        ("All tracked data".to_string(), db.stats(None)?)
    };

    print_stats(&label, &stats);
    Ok(())
}

pub fn locations_command() -> Result<()> {
    let active_database = get_database_path()?;
    let global_database = get_global_database_path()?;

    println!("Active database: {}", active_database.display());
    if let Some(workspace) = get_contrack_dir() {
        println!("Project workspace: {}", workspace.display());
    } else {
        println!("Project workspace: not active in the current directory tree");
    }
    println!("Global fallback: {}", global_database.display());
    Ok(())
}

fn refresh_inner(repo_selector: Option<String>, all: bool) -> Result<()> {
    let db = Database::open()?;
    let repositories = if all {
        db.list_repositories()?
    } else {
        vec![resolve_repository(&db, repo_selector.as_deref())?]
    };

    if repositories.is_empty() {
        return Err(anyhow!(
            "No tracked repositories. Start with `contrack repo add .`."
        ));
    }

    for repository in repositories {
        let commits = git::extract_commits(Path::new(&repository.local_path))?;
        let count = db.import_commits(repository.id, &commits)?;
        let contributions = db.list_contributions(Some(repository.id))?;
        let imported = db.list_all_commits_for_repository(repository.id)?;
        let matched_refs = contributions
            .iter()
            .map(|contribution| build_evidence(contribution, &imported))
            .map(|evidence| evidence.key_commits.len() + evidence.related_commits.len())
            .sum::<usize>();

        println!("Refreshed `{}`", repository.slug);
        println!("  imported rows: {}", count);
        println!("  commits seen: {}", commits.len());
        println!("  matched contribution refs: {}", matched_refs);
    }

    Ok(())
}

fn resolve_repository(db: &Database, selector: Option<&str>) -> Result<TrackedRepository> {
    if let Some(selector) = selector {
        return db
            .get_repository(selector)?
            .with_context(|| format!("No tracked repository matched '{selector}'."));
    }

    infer_repository_from_context(db)
}

fn should_infer_repo(db: &Database, selector: Option<&str>) -> Result<bool> {
    if selector.is_some() {
        return Ok(true);
    }

    let repositories = db.list_repositories()?;
    Ok(!repositories.is_empty())
}

fn infer_repository_from_context(db: &Database) -> Result<TrackedRepository> {
    let repositories = db.list_repositories()?;

    if repositories.is_empty() {
        return Err(anyhow!(
            "No tracked repositories. Start with `contrack repo add .`."
        ));
    }

    let current_dir = std::env::current_dir().context("Failed to resolve the current directory")?;
    let current_dir = canonicalize_path(&current_dir).unwrap_or(current_dir);

    let mut path_matches = repositories
        .iter()
        .filter(|repository| current_dir.starts_with(Path::new(&repository.local_path)))
        .cloned()
        .collect::<Vec<_>>();
    path_matches.sort_by_key(|repository| repository.local_path.len());
    if let Some(repository) = path_matches.pop() {
        return Ok(repository);
    }

    if let Ok(metadata) = git::inspect_repository(&current_dir) {
        for repository in &repositories {
            if repository.local_path == metadata.root_path.display().to_string() {
                return Ok(repository.clone());
            }
            if repository.remote_url == metadata.normalized_remote_url {
                return Ok(repository.clone());
            }
        }
    }

    if repositories.len() == 1 {
        return Ok(repositories.into_iter().next().expect("single repository"));
    }

    Err(anyhow!(
        "Could not infer a tracked repository from the current directory. Pass `--repo <slug>`.",
    ))
}

fn validate_priority(priority: u8) -> Result<()> {
    if (1..=5).contains(&priority) {
        Ok(())
    } else {
        Err(anyhow!("Priority must be between 1 and 5."))
    }
}

fn sanitize_refs(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn sanitize_lines(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn build_evidence(contribution: &Contribution, commits: &[StoredCommit]) -> ContributionEvidence {
    let (key_commits, unresolved_key_refs) =
        resolve_commit_refs(&contribution.key_commit_refs, commits);
    let (related_commits, unresolved_related_refs) =
        resolve_commit_refs(&contribution.related_commit_refs, commits);

    ContributionEvidence {
        contribution: contribution.clone(),
        key_commits,
        related_commits,
        unresolved_key_refs,
        unresolved_related_refs,
    }
}

fn resolve_commit_refs(
    refs: &[String],
    commits: &[StoredCommit],
) -> (Vec<StoredCommit>, Vec<String>) {
    let mut matched_hashes = HashSet::new();
    let mut matched = Vec::new();
    let mut unresolved = Vec::new();

    for reference in refs {
        let exact_match = commits
            .iter()
            .find(|commit| commit.hash.eq_ignore_ascii_case(reference));
        if let Some(commit) = exact_match {
            if matched_hashes.insert(commit.hash.clone()) {
                matched.push(commit.clone());
            }
            continue;
        }

        let prefix_matches = commits
            .iter()
            .filter(|commit| commit.hash.starts_with(reference))
            .collect::<Vec<_>>();

        if prefix_matches.len() == 1 {
            let commit = prefix_matches[0];
            if matched_hashes.insert(commit.hash.clone()) {
                matched.push(commit.clone());
            }
        } else {
            unresolved.push(reference.clone());
        }
    }

    (matched, unresolved)
}

fn print_commit_matches(title: &str, commits: &[StoredCommit], unresolved_refs: &[String]) {
    if commits.is_empty() && unresolved_refs.is_empty() {
        return;
    }

    println!();
    println!("{}:", title);

    for commit in commits {
        println!(
            "- {} {} by {} (+{}, -{}, {} files)",
            shorten_hash(&commit.hash),
            commit.summary,
            commit.author_name,
            commit.lines_added,
            commit.lines_deleted,
            commit.files_changed.len(),
        );
    }

    for unresolved in unresolved_refs {
        println!("- {} (not imported yet)", unresolved);
    }
}

fn print_commit_line(commit: &StoredCommit) {
    println!(
        "{}  {}  {}  {} (+{}, -{})",
        shorten_hash(&commit.hash),
        commit.committed_at,
        commit.author_name,
        commit.summary,
        commit.lines_added,
        commit.lines_deleted
    );
}

fn print_stats(label: &str, stats: &DatabaseStats) {
    println!("{}", label);
    println!("Repositories: {}", stats.repositories);
    println!("Contributions: {}", stats.contributions);
    println!("Commits: {}", stats.commits);
}
