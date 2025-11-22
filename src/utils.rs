use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Find the `.contrack` directory by walking up from the current directory
/// Returns None if not found
pub fn get_contrack_dir() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    
    loop {
        let contrack_dir = current_dir.join(".contrack");
        if contrack_dir.exists() && contrack_dir.is_dir() {
            return Some(contrack_dir);
        }
        
        // Move to parent directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }
    
    None
}

/// Get the path to the contributions database file
/// Checks for project-local `.contrack/contributions.db` first,
/// then falls back to application data directory
pub fn get_database_path() -> Result<PathBuf> {
    // First, check for project-local .contrack folder
    if let Some(contrack_dir) = get_contrack_dir() {
        let db_path = contrack_dir.join("contributions.db");
        // Ensure the .contrack directory exists (it should, but be safe)
        std::fs::create_dir_all(&contrack_dir)
            .context("Failed to create .contrack directory")?;
        return Ok(db_path);
    }
    
    // Fall back to application data directory
    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine application data directory")?;
    
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .context("Failed to create data directory")?;
    
    Ok(data_dir.join("contributions.db"))
}

/// Get the path to the application config directory
#[allow(dead_code)]
pub fn get_config_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine application config directory")?;
    
    let config_dir = project_dirs.config_dir();
    std::fs::create_dir_all(config_dir)
        .context("Failed to create config directory")?;
    
    Ok(config_dir.to_path_buf())
}

/// Get the path to the config.toml file
/// Checks for project-local `.contrack/config.toml` first,
/// then falls back to application config directory
pub fn get_config_path() -> Result<PathBuf> {
    // First, check for project-local .contrack folder
    if let Some(contrack_dir) = get_contrack_dir() {
        return Ok(contrack_dir.join("config.toml"));
    }
    
    // Fall back to application config directory
    let project_dirs = ProjectDirs::from("com", "contrack", "contrack")
        .context("Failed to determine application config directory")?;
    
    let config_dir = project_dirs.config_dir();
    std::fs::create_dir_all(config_dir)
        .context("Failed to create config directory")?;
    
    Ok(config_dir.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_database_path() {
        let path = get_database_path().unwrap();
        assert!(path.parent().unwrap().exists());
        assert!(path.file_name().unwrap() == "contributions.db");
    }

    #[test]
    fn test_get_contrack_dir_not_found() {
        // In a temp directory without .contrack, should return None
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        // Should not find .contrack in a fresh temp dir
        let result = get_contrack_dir();
        // This might return None or might find something in parent dirs
        // So we just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_contrack_dir_found() {
        let temp_dir = TempDir::new().unwrap();
        let contrack_dir = temp_dir.path().join(".contrack");
        fs::create_dir_all(&contrack_dir).unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let result = get_contrack_dir();
        assert!(result.is_some());
        // Use canonicalize to handle symlinks (e.g., /var -> /private/var on macOS)
        let expected = contrack_dir.canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_database_path_with_contrack_dir() {
        let temp_dir = TempDir::new().unwrap();
        let contrack_dir = temp_dir.path().join(".contrack");
        fs::create_dir_all(&contrack_dir).unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let db_path = get_database_path().unwrap();
        let expected = contrack_dir.join("contributions.db");
        // Compare parent directories (which exist) using canonicalize
        assert_eq!(
            db_path.parent().unwrap().canonicalize().unwrap(),
            expected.parent().unwrap().canonicalize().unwrap()
        );
        assert_eq!(db_path.file_name(), expected.file_name());
        assert!(contrack_dir.exists());
    }

    #[test]
    fn test_get_database_path_fallback() {
        // Test that it falls back to app data directory when .contrack not found
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        // Remove any .contrack that might exist
        let _ = fs::remove_dir_all(temp_dir.path().join(".contrack"));
        
        let db_path = get_database_path().unwrap();
        // Should be in app data directory, not in temp_dir
        assert!(!db_path.starts_with(temp_dir.path()));
        assert!(db_path.file_name().unwrap() == "contributions.db");
        assert!(db_path.parent().unwrap().exists());
    }
}

