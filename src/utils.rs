use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Get the path to the contributions database file
pub fn get_database_path() -> Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_path() {
        let path = get_database_path().unwrap();
        assert!(path.parent().unwrap().exists());
        assert!(path.file_name().unwrap() == "contributions.db");
    }
}

