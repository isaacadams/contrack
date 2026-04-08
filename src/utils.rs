use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

pub const DATABASE_FILE_NAME: &str = "contrack.db";
pub const WORKSPACE_DIR_NAME: &str = ".contrack";

pub fn initialize_local_workspace() -> Result<PathBuf> {
    let workspace = std::env::current_dir()
        .context("Failed to resolve the current directory")?
        .join(WORKSPACE_DIR_NAME);
    std::fs::create_dir_all(&workspace)
        .with_context(|| format!("Failed to create {}", workspace.display()))?;
    Ok(workspace)
}

pub fn get_contrack_dir() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        let candidate = current.join(WORKSPACE_DIR_NAME);
        if candidate.is_dir() {
            return Some(candidate);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}

pub fn get_database_path() -> Result<PathBuf> {
    if let Some(workspace) = get_contrack_dir() {
        return Ok(workspace.join(DATABASE_FILE_NAME));
    }

    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine the global contrack data directory")?;
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("Failed to create {}", data_dir.display()))?;

    Ok(data_dir.join(DATABASE_FILE_NAME))
}

pub fn get_global_database_path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine the global contrack data directory")?;
    Ok(project_dirs.data_dir().join(DATABASE_FILE_NAME))
}

pub fn canonicalize_path(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", path.display()))
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            slug.push('-');
            previous_was_separator = true;
        }
    }

    slug.trim_matches('-').to_string()
}

pub fn shorten_hash(hash: &str) -> &str {
    let len = hash.len().min(8);
    &hash[..len]
}

pub fn join_lines(lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes_text() {
        assert_eq!(slugify("Owner/Repo Name"), "owner-repo-name");
        assert_eq!(slugify("___Hello___World___"), "hello-world");
    }

    #[test]
    fn shorten_hash_handles_short_hashes() {
        assert_eq!(shorten_hash("abc"), "abc");
        assert_eq!(shorten_hash("1234567890"), "12345678");
    }
}
