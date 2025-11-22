use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub organization: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub organizations: HashMap<String, Organization>,
    #[serde(default)]
    pub repositories: HashMap<String, RepositoryConfig>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            organizations: HashMap::new(),
            repositories: HashMap::new(),
        }
    }

    pub fn from_toml(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        Ok(config)
    }

    pub fn to_toml(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        std::fs::write(path, toml_string)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.organizations.is_empty());
        assert!(config.repositories.is_empty());
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let mut config = Config::new();
        
        config.organizations.insert(
            "org1".to_string(),
            Organization {
                name: "Organization 1".to_string(),
                description: Some("Test org".to_string()),
            },
        );

        config.repositories.insert(
            "https://github.com/org1/repo1".to_string(),
            RepositoryConfig {
                organization: "org1".to_string(),
                name: "repo1".to_string(),
                description: Some("Test repo".to_string()),
            },
        );

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        // Serialize
        config.to_toml(&path).unwrap();

        // Deserialize
        let loaded = Config::from_toml(&path).unwrap();
        assert_eq!(loaded.organizations.len(), 1);
        assert_eq!(loaded.repositories.len(), 1);
        assert_eq!(loaded.organizations.get("org1").unwrap().name, "Organization 1");
        assert_eq!(loaded.repositories.get("https://github.com/org1/repo1").unwrap().name, "repo1");
    }

    #[test]
    fn test_config_empty_serialize() {
        let config = Config::new();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        config.to_toml(&path).unwrap();
        let loaded = Config::from_toml(&path).unwrap();
        assert!(loaded.organizations.is_empty());
        assert!(loaded.repositories.is_empty());
    }
}

