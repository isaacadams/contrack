use anyhow::{Context, Result};
use git2::Repository;
use std::path::{Path, PathBuf};

use crate::utils::canonicalize_path;

#[derive(Debug, Clone)]
pub struct GitRepositoryMetadata {
    pub root_path: PathBuf,
    pub inferred_name: String,
    pub normalized_remote_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub committed_at: String,
    pub summary: String,
    pub body: Option<String>,
    pub files_changed: Vec<String>,
    pub lines_added: i64,
    pub lines_deleted: i64,
}

pub fn inspect_repository(path: &Path) -> Result<GitRepositoryMetadata> {
    let repo = Repository::discover(path)
        .with_context(|| format!("Failed to find a git repository from {}", path.display()))?;
    let root_path = repo
        .workdir()
        .map(canonicalize_path)
        .transpose()?
        .or_else(|| repo.path().parent().map(Path::to_path_buf))
        .context("Failed to determine the repository root")?;

    let normalized_remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|remote| remote.url().map(normalize_remote_url));

    let inferred_name = normalized_remote_url
        .as_deref()
        .and_then(repository_name_from_remote)
        .unwrap_or_else(|| {
            root_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("repository")
                .to_string()
        });

    Ok(GitRepositoryMetadata {
        root_path,
        inferred_name,
        normalized_remote_url,
    })
}

pub fn extract_commits(path: &Path) -> Result<Vec<GitCommit>> {
    let repo = Repository::open(path)
        .with_context(|| format!("Failed to open git repository at {}", path.display()))?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let author = commit.author();
        let summary = commit
            .summary()
            .unwrap_or("No commit message")
            .trim()
            .to_string();
        let body = commit
            .body()
            .map(str::trim)
            .filter(|body| !body.is_empty())
            .map(str::to_string);

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|parent| parent.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut files_changed = Vec::new();
        let mut lines_added = 0_i64;
        let mut lines_deleted = 0_i64;

        diff.foreach(
            &mut |delta, _| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|value| value.to_string_lossy().to_string());
                if let Some(path) = path {
                    files_changed.push(path);
                }
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' => lines_added += 1,
                    '-' => lines_deleted += 1,
                    _ => {}
                }
                true
            }),
        )?;

        let committed_at =
            chrono::DateTime::<chrono::Utc>::from_timestamp(commit.time().seconds(), 0)
                .unwrap_or_default()
                .to_rfc3339();

        commits.push(GitCommit {
            hash: oid.to_string(),
            author_name: author.name().unwrap_or("Unknown Author").to_string(),
            author_email: author.email().map(str::to_string),
            committed_at,
            summary,
            body,
            files_changed,
            lines_added,
            lines_deleted,
        });
    }

    Ok(commits)
}

fn repository_name_from_remote(remote_url: &str) -> Option<String> {
    remote_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
}

pub fn normalize_remote_url(remote_url: &str) -> String {
    let trimmed = remote_url
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git");

    if let Some(rest) = trimmed.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            return format!("https://{host}/{}", path.trim_start_matches('/'));
        }
    }

    if let Some(rest) = trimmed.strip_prefix("ssh://") {
        let without_user = rest.split('@').nth(1).unwrap_or(rest);
        return format!("https://{}", without_user.trim_start_matches('/'));
    }

    if let Some(rest) = trimmed.strip_prefix("http://") {
        return format!("https://{rest}");
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_remote_url;

    #[test]
    fn normalize_https_remote() {
        assert_eq!(
            normalize_remote_url("https://github.com/isaacadams/contrack.git"),
            "https://github.com/isaacadams/contrack"
        );
    }

    #[test]
    fn normalize_ssh_remote() {
        assert_eq!(
            normalize_remote_url("git@github.com:isaacadams/contrack.git"),
            "https://github.com/isaacadams/contrack"
        );
    }

    #[test]
    fn normalize_ssh_protocol_remote() {
        assert_eq!(
            normalize_remote_url("ssh://git@github.com/isaacadams/contrack.git"),
            "https://github.com/isaacadams/contrack"
        );
    }
}
