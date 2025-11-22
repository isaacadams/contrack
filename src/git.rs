use anyhow::{Context, Result};
use git2::{Repository, Oid};
use std::path::PathBuf;

use crate::database::Commit;

pub fn extract_commits_from_repo(repo_path: &PathBuf) -> Result<Vec<Commit>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at {:?}", repo_path))?;

    // Get remote URL for repository identification
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let mut commits = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    for oid in revwalk {
        let oid = oid?;
        let commit_obj = repo.find_commit(oid)?;
        
        let author = commit_obj.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("unknown@example.com").to_string();

        let time = commit_obj.time();
        let date = chrono::DateTime::<chrono::Utc>::from_timestamp(time.seconds(), 0)
            .unwrap_or_default()
            .to_rfc3339();

        let message = commit_obj.message().unwrap_or("").to_string();
        let hash = oid.to_string();

        // Get diff stats
        let (lines_added, lines_deleted, files_changed) = if let Ok(tree) = commit_obj.tree() {
            let parent_tree = commit_obj
                .parent(0)
                .ok()
                .and_then(|p| p.tree().ok());
            
            let diff = repo.diff_tree_to_tree(
                parent_tree.as_ref(),
                Some(&tree),
                None,
            )?;

            let mut added = 0;
            let mut deleted = 0;
            let mut files = Vec::new();

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        files.push(path.to_string_lossy().to_string());
                    }
                    true
                },
                None,
                None,
                Some(&mut |_delta, _hunk, line| {
                    let origin = line.origin();
                    if origin == '+' {
                        added += 1;
                    } else if origin == '-' {
                        deleted += 1;
                    }
                    true
                }),
            )?;

            (Some(added), Some(deleted), files)
        } else {
            (None, None, Vec::new())
        };

        commits.push(Commit {
            hash,
            repository_url: remote_url.clone(),
            contribution_id: None, // Will be set later
            author: author_name,
            author_email,
            date,
            message,
            files_changed,
            lines_added,
            lines_deleted,
        });
    }

    Ok(commits)
}

pub fn get_commit_details(commit_hash: &str, repo_path: &PathBuf) -> Result<Option<Commit>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repository at {:?}", repo_path))?;

    let oid = Oid::from_str(commit_hash)
        .with_context(|| format!("Invalid commit hash: {}", commit_hash))?;

    let commit_obj = repo.find_commit(oid)?;

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let author = commit_obj.author();
    let author_name = author.name().unwrap_or("Unknown").to_string();
    let author_email = author.email().unwrap_or("unknown@example.com").to_string();

    let time = commit_obj.time();
    let date = chrono::DateTime::<chrono::Utc>::from_timestamp(time.seconds(), 0)
        .unwrap_or_default()
        .to_rfc3339();

    let message = commit_obj.message().unwrap_or("").to_string();

    // Get diff stats (simplified)
    let (lines_added, lines_deleted, files_changed) = if let Ok(tree) = commit_obj.tree() {
        let parent_tree = commit_obj
            .parent(0)
            .ok()
            .and_then(|p| p.tree().ok());
        
        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            None,
        )?;

        let mut added = 0;
        let mut deleted = 0;
        let mut files = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                let origin = line.origin();
                if origin == '+' {
                    added += 1;
                } else if origin == '-' {
                    deleted += 1;
                }
                true
            }),
        )?;

        (Some(added), Some(deleted), files)
    } else {
        (None, None, Vec::new())
    };

    Ok(Some(Commit {
        hash: commit_hash.to_string(),
        repository_url: remote_url,
        contribution_id: None,
        author: author_name,
        author_email,
        date,
        message,
        files_changed,
        lines_added,
        lines_deleted,
    }))
}

