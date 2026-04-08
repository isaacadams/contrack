use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::database::{
    CommitAuthorSummary, Contribution, Database, DatabaseStats, NewContribution, NewRepository,
    RepositoryStatus, StoredCommit, TrackedRepository,
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

pub fn repo_list_command(json_output: bool) -> Result<()> {
    let db = Database::open()?;
    let repositories = db.list_repositories()?;

    if json_output {
        return print_json(&repositories);
    }

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

pub fn repo_status_command(repo_selector: Option<String>, json_output: bool) -> Result<()> {
    let db = Database::open()?;
    let repository_id = match repo_selector.as_deref() {
        Some(_) => Some(resolve_repository(&db, repo_selector.as_deref())?.id),
        None => None,
    };
    let current_repository_id = infer_repository_from_context(&db).ok().map(|repo| repo.id);
    let statuses = db.list_repository_statuses(repository_id)?;

    if json_output {
        let payload = statuses
            .into_iter()
            .map(|status| {
                json!({
                    "repository": status.repository,
                    "contributions": status.contributions,
                    "commits": status.commits,
                    "latest_commit_at": status.latest_commit_at,
                    "latest_imported_at": status.latest_imported_at,
                    "current_context": current_repository_id == Some(status.repository.id),
                })
            })
            .collect::<Vec<_>>();
        return print_json(&payload);
    }

    if statuses.is_empty() {
        println!("No tracked repositories yet.");
        println!("Start with: contrack repo add .");
        return Ok(());
    }

    for status in statuses {
        print_repository_status(&status, current_repository_id == Some(status.repository.id));
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
        "Removed repository `{}` and its stored contributions, commits, and PR evidence.",
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
    covered_prs: Vec<i64>,
    technical_details: Vec<String>,
    resume_bullets: Vec<String>,
    rationale: Option<String>,
    confidence: Option<String>,
    status: String,
) -> Result<()> {
    validate_priority(priority)?;
    validate_status(&status)?;
    validate_confidence(confidence.as_deref())?;
    let db = Database::open()?;
    let repository = resolve_repository(&db, repo_selector.as_deref())?;

    let contribution = db.add_contribution(&NewContribution {
        repository_id: repository.id,
        name: name.clone(),
        overview,
        description,
        category,
        priority,
        status,
        confidence,
        rationale: sanitize_optional_line(rationale),
        covered_prs: sanitize_prs(covered_prs),
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
    covered_prs: Option<Vec<i64>>,
    technical_details: Option<Vec<String>>,
    resume_bullets: Option<Vec<String>>,
    rationale: Option<String>,
    confidence: Option<String>,
    status: Option<String>,
    clear_key_commits: bool,
    clear_related_commits: bool,
    clear_covered_prs: bool,
    clear_technical_details: bool,
    clear_resume_bullets: bool,
    clear_rationale: bool,
    clear_confidence: bool,
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
    if let Some(status) = status {
        validate_status(&status)?;
        contribution.status = status;
    }
    if clear_confidence {
        contribution.confidence = None;
    } else if let Some(confidence) = confidence {
        validate_confidence(Some(&confidence))?;
        contribution.confidence = Some(confidence);
    }
    if clear_rationale {
        contribution.rationale = None;
    } else if let Some(rationale) = rationale {
        contribution.rationale = sanitize_optional_line(Some(rationale));
    }
    if clear_covered_prs {
        contribution.covered_prs.clear();
    } else if let Some(covered_prs) = covered_prs {
        contribution.covered_prs = sanitize_prs(covered_prs);
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

pub fn contribution_list_command(repo_selector: Option<String>, json_output: bool) -> Result<()> {
    let db = Database::open()?;
    let repository_id = match repo_selector.as_deref() {
        Some(_) => Some(resolve_repository(&db, repo_selector.as_deref())?.id),
        None => None,
    };

    let contributions = db.list_contributions(repository_id, None)?;
    if json_output {
        return print_json(&contributions);
    }

    if contributions.is_empty() {
        println!("No contributions found.");
        return Ok(());
    }

    for contribution in contributions {
        println!(
            "{}  [{}] p{} {} {}",
            contribution.id,
            contribution.repository_slug,
            contribution.priority,
            contribution.status,
            contribution.name
        );
        println!(
            "  confidence: {} | key commits: {} | related commits: {} | covered PRs: {}",
            contribution.confidence.as_deref().unwrap_or("unset"),
            contribution.key_commit_refs.len(),
            contribution.related_commit_refs.len(),
            contribution.covered_prs.len(),
        );
        println!("  {}", contribution.overview);
    }

    Ok(())
}

pub fn contribution_show_command(selector: String, json_output: bool) -> Result<()> {
    let db = Database::open()?;
    let contribution = db
        .get_contribution(&selector)?
        .with_context(|| format!("No contribution matched '{selector}'."))?;
    let commits = db.list_all_commits_for_repository(contribution.repository_id)?;
    let evidence = build_evidence(&contribution, &commits);

    if json_output {
        return print_json(&json!({
            "contribution": contribution,
            "evidence": evidence,
        }));
    }

    println!("{} ({})", contribution.name, contribution.id);
    println!("Repository: {}", contribution.repository_slug);
    println!("Category: {}", contribution.category);
    println!("Priority: {}", contribution.priority);
    println!("Status: {}", contribution.status);
    println!(
        "Confidence: {}",
        contribution.confidence.as_deref().unwrap_or("unset")
    );
    if !contribution.covered_prs.is_empty() {
        println!("Covered PRs: {:?}", contribution.covered_prs);
    }
    if let Some(rationale) = &contribution.rationale {
        println!("Rationale: {}", rationale);
    }
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

pub fn contribution_merge_command(
    primary_selector: String,
    secondary_selector: String,
) -> Result<()> {
    let db = Database::open()?;
    let mut primary = db
        .get_contribution(&primary_selector)?
        .with_context(|| format!("No contribution matched '{primary_selector}'."))?;
    let secondary = db
        .get_contribution(&secondary_selector)?
        .with_context(|| format!("No contribution matched '{secondary_selector}'."))?;

    if primary.id == secondary.id {
        return Err(anyhow!("Cannot merge a contribution into itself."));
    }
    if primary.repository_id != secondary.repository_id {
        return Err(anyhow!(
            "Can only merge contributions within the same repository."
        ));
    }

    primary.priority = primary.priority.max(secondary.priority);
    primary.status = merge_status(&primary.status, &secondary.status);
    primary.confidence = merge_confidence(primary.confidence.clone(), secondary.confidence.clone());
    primary.rationale = merge_optional_text(primary.rationale.clone(), secondary.rationale.clone());
    primary.covered_prs = merge_numbers(&primary.covered_prs, &secondary.covered_prs);
    primary.key_commit_refs = merge_strings(&primary.key_commit_refs, &secondary.key_commit_refs);
    primary.related_commit_refs =
        merge_strings(&primary.related_commit_refs, &secondary.related_commit_refs);
    primary.technical_details =
        merge_strings(&primary.technical_details, &secondary.technical_details);
    primary.resume_bullets = merge_strings(&primary.resume_bullets, &secondary.resume_bullets);
    primary.description = format!(
        "{}\n\n{}",
        primary.description.trim(),
        secondary.description.trim()
    );

    db.update_contribution(&primary)?;
    db.delete_contribution(secondary.id)?;

    println!(
        "Merged contribution {} into {} ({})",
        secondary.id, primary.id, primary.name
    );
    Ok(())
}

pub fn contribution_link_pr_command(selector: String, prs: Vec<i64>, replace: bool) -> Result<()> {
    let db = Database::open()?;
    let mut contribution = db
        .get_contribution(&selector)?
        .with_context(|| format!("No contribution matched '{selector}'."))?;
    let prs = sanitize_prs(prs);

    if prs.is_empty() {
        return Err(anyhow!("Pass at least one PR number to link."));
    }

    contribution.covered_prs = if replace {
        prs
    } else {
        merge_numbers(&contribution.covered_prs, &prs)
    };

    db.update_contribution(&contribution)?;
    println!(
        "Linked {} PR(s) to contribution {} ({})",
        contribution.covered_prs.len(),
        contribution.id,
        contribution.name
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
    json_output: bool,
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

        if json_output {
            return print_json(&matched);
        }

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
    if json_output {
        return print_json(&commits);
    }

    if commits.is_empty() {
        println!("No imported commits for repository `{}`.", repository.slug);
        return Ok(());
    }

    for commit in commits {
        print_commit_line(&commit);
    }
    Ok(())
}

pub fn commit_authors_command(
    repo_selector: Option<String>,
    limit: usize,
    json_output: bool,
) -> Result<()> {
    let db = Database::open()?;
    let repository = match repo_selector.as_deref() {
        Some(_) => Some(resolve_repository(&db, repo_selector.as_deref())?),
        None => None,
    };
    let authors = db.list_commit_authors(repository.as_ref().map(|repo| repo.id), limit)?;

    if json_output {
        return print_json(&authors);
    }

    if authors.is_empty() {
        if let Some(repository) = repository {
            println!("No imported commits for repository `{}`.", repository.slug);
        } else {
            println!("No imported commits found.");
        }
        return Ok(());
    }

    if let Some(repository) = &repository {
        println!("Authors for repository `{}`", repository.slug);
    } else {
        println!("Authors across tracked repositories");
    }

    for author in authors {
        print_author_summary(&author);
    }

    Ok(())
}

pub fn generate_markdown_command(
    repo_selector: Option<String>,
    style: MarkdownStyle,
    output: Option<PathBuf>,
    include: Option<String>,
    status: Option<String>,
    json_output: bool,
) -> Result<()> {
    if let Some(status) = status.as_deref() {
        validate_status(status)?;
    }

    let db = Database::open()?;
    let repository = resolve_repository(&db, repo_selector.as_deref())?;
    let include_ids = parse_include_ids(include.as_deref())?;
    let mut contributions = db.list_contributions(Some(repository.id), status.as_deref())?;

    if !include_ids.is_empty() {
        contributions.retain(|contribution| include_ids.contains(&contribution.id));
    }

    if contributions.is_empty() {
        return Err(anyhow!(
            "No contributions found for repository `{}` after filtering.",
            repository.slug
        ));
    }

    let commits = db.list_all_commits_for_repository(repository.id)?;
    let items = contributions
        .iter()
        .map(|contribution| build_evidence(contribution, &commits))
        .collect::<Vec<_>>();
    let rendered = markdown::render_markdown(&repository, &items, style);

    if json_output {
        return print_json(&json!({
            "repository": repository,
            "style": match style { MarkdownStyle::Resume => "resume", MarkdownStyle::Portfolio => "portfolio" },
            "markdown": rendered,
        }));
    }

    if let Some(output) = output {
        fs::write(&output, rendered)
            .with_context(|| format!("Failed to write {}", output.display()))?;
        println!("Wrote Markdown to {}", output.display());
    } else {
        print!("{}", rendered);
    }
    Ok(())
}

pub fn stats_command(repo_selector: Option<String>, json_output: bool) -> Result<()> {
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

    if json_output {
        return print_json(&json!({ "label": label, "stats": stats }));
    }

    print_stats(&label, &stats);
    Ok(())
}

pub fn locations_command(json_output: bool) -> Result<()> {
    let active_database = get_database_path()?;
    let global_database = get_global_database_path()?;
    let project_workspace = get_contrack_dir().map(|path| path.display().to_string());

    if json_output {
        return print_json(&json!({
            "active_database": active_database.display().to_string(),
            "project_workspace": project_workspace,
            "global_fallback": global_database.display().to_string(),
        }));
    }

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
        let contributions = db.list_contributions(Some(repository.id), None)?;
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
            .with_context(|| repository_not_found_message(selector));
    }

    infer_repository_from_context(db)
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

        return Err(anyhow!(
            "Current directory is a git repository, but it is not tracked in Contrack. Add it with `contrack repo add \"{}\" --slug <slug>`.",
            metadata.root_path.display()
        ));
    }

    Err(anyhow!(
        "Could not infer a tracked repository from the current directory. Pass `--repo <slug>`.",
    ))
}

fn repository_not_found_message(selector: &str) -> String {
    format!(
        "No tracked repository matched '{selector}'. Try `contrack repo add <path> --slug <slug>` first."
    )
}

fn validate_priority(priority: u8) -> Result<()> {
    if (1..=5).contains(&priority) {
        Ok(())
    } else {
        Err(anyhow!("Priority must be between 1 and 5."))
    }
}

fn validate_status(status: &str) -> Result<()> {
    match status {
        "draft" | "accepted" => Ok(()),
        _ => Err(anyhow!("Status must be one of: draft, accepted.")),
    }
}

fn validate_confidence(confidence: Option<&str>) -> Result<()> {
    match confidence {
        None => Ok(()),
        Some("high" | "medium" | "low") => Ok(()),
        Some(_) => Err(anyhow!("Confidence must be one of: high, medium, low.")),
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

fn sanitize_prs(values: Vec<i64>) -> Vec<i64> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(*value))
        .collect()
}

fn sanitize_lines(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn sanitize_optional_line(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
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

fn parse_include_ids(raw: Option<&str>) -> Result<HashSet<i64>> {
    let mut ids = HashSet::new();
    let Some(raw) = raw else {
        return Ok(ids);
    };

    for token in raw.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        ids.insert(
            trimmed
                .parse::<i64>()
                .with_context(|| format!("Invalid contribution id '{trimmed}'."))?,
        );
    }
    Ok(ids)
}

fn merge_status(primary: &str, secondary: &str) -> String {
    if primary == "accepted" || secondary == "accepted" {
        "accepted".to_string()
    } else {
        "draft".to_string()
    }
}

fn merge_confidence(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    let rank = |value: &str| match value {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    };

    match (primary, secondary) {
        (Some(left), Some(right)) => Some(if rank(&left) >= rank(&right) {
            left
        } else {
            right
        }),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn merge_optional_text(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    match (primary, secondary) {
        (Some(left), Some(right)) if left.trim() == right.trim() => Some(left),
        (Some(left), Some(right)) => Some(format!("{}\n\n{}", left.trim(), right.trim())),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn merge_strings(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    primary
        .iter()
        .chain(secondary.iter())
        .filter(|value| seen.insert((*value).clone()))
        .cloned()
        .collect()
}

fn merge_numbers(primary: &[i64], secondary: &[i64]) -> Vec<i64> {
    let mut seen = HashSet::new();
    primary
        .iter()
        .chain(secondary.iter())
        .filter(|value| seen.insert(**value))
        .copied()
        .collect()
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
        commit.lines_deleted,
    );
}

fn print_stats(label: &str, stats: &DatabaseStats) {
    println!("{}", label);
    println!("Repositories: {}", stats.repositories);
    println!("Contributions: {}", stats.contributions);
    println!("Commits: {}", stats.commits);
}

fn print_repository_status(status: &RepositoryStatus, current_context: bool) {
    let marker = if current_context { "  [current]" } else { "" };
    println!(
        "{}  {}{}",
        status.repository.slug, status.repository.name, marker
    );
    println!("  path: {}", status.repository.local_path);
    if let Some(remote_url) = &status.repository.remote_url {
        println!("  remote: {}", remote_url);
    }
    println!(
        "  contributions: {} | commits: {}",
        status.contributions, status.commits
    );
    println!(
        "  latest commit: {} | latest refresh: {}",
        status.latest_commit_at.as_deref().unwrap_or("none"),
        status.latest_imported_at.as_deref().unwrap_or("none")
    );
}

fn print_author_summary(author: &CommitAuthorSummary) {
    match &author.author_email {
        Some(email) => println!(
            "{} <{}>  commits: {}  latest: {}",
            author.author_name, email, author.commit_count, author.latest_commit_at
        ),
        None => println!(
            "{}  commits: {}  latest: {}",
            author.author_name, author.commit_count, author.latest_commit_at
        ),
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
