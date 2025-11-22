use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::utils::get_database_path;

type AgentRule = (String, String, i32, Option<String>);
type PromptInfo = (String, String, Option<String>, Option<String>);

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub url: String,
    pub organization: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Contribution {
    pub id: Option<i64>,
    pub repository_url: String,
    pub name: String,
    pub overview: String,
    pub description: String,
    pub key_commits: Vec<String>,
    pub related_commits: Vec<String>,
    pub technical_details: HashMap<String, serde_json::Value>,
    pub resume_bullets: Vec<String>,
    pub category: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub repository_url: String,
    pub contribution_id: Option<i64>,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub message: String,
    pub files_changed: Vec<String>,
    pub lines_added: Option<i32>,
    pub lines_deleted: Option<i32>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let db_path = get_database_path()?;
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;
        
        let db = Database { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        // Repositories table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS repositories (
                repository_url TEXT PRIMARY KEY,
                organization TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Contributions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS contributions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repository_url TEXT NOT NULL,
                name TEXT NOT NULL,
                overview TEXT,
                description TEXT,
                key_commits TEXT,
                related_commits TEXT,
                technical_details TEXT,
                resume_bullets TEXT,
                category TEXT,
                priority INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repository_url) REFERENCES repositories(repository_url),
                UNIQUE(repository_url, name)
            )",
            [],
        )?;

        // Commits table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commits (
                commit_hash TEXT PRIMARY KEY,
                repository_url TEXT NOT NULL,
                contribution_id INTEGER,
                author TEXT NOT NULL,
                author_email TEXT,
                date TEXT NOT NULL,
                message TEXT,
                files_changed TEXT,
                lines_added INTEGER,
                lines_deleted INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repository_url) REFERENCES repositories(repository_url),
                FOREIGN KEY (contribution_id) REFERENCES contributions(id)
            )",
            [],
        )?;

        // Agent rules table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                instruction TEXT NOT NULL,
                priority INTEGER DEFAULT 0,
                category TEXT,
                examples TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Prompts table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                prompt_text TEXT NOT NULL,
                description TEXT,
                category TEXT,
                variables TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Loadouts table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS loadouts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_default INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Loadout prompts junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS loadout_prompts (
                loadout_id INTEGER NOT NULL,
                prompt_id INTEGER NOT NULL,
                PRIMARY KEY (loadout_id, prompt_id),
                FOREIGN KEY (loadout_id) REFERENCES loadouts(id) ON DELETE CASCADE,
                FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Loadout rules junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS loadout_rules (
                loadout_id INTEGER NOT NULL,
                rule_id INTEGER NOT NULL,
                PRIMARY KEY (loadout_id, rule_id),
                FOREIGN KEY (loadout_id) REFERENCES loadouts(id) ON DELETE CASCADE,
                FOREIGN KEY (rule_id) REFERENCES agent_rules(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_contributions_repo ON contributions(repository_url)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_commits_repo ON commits(repository_url)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_commits_contribution ON commits(contribution_id)",
            [],
        )?;

        // Initialize agent rules if they don't exist
        self.initialize_agent_rules()?;
        self.initialize_prompts()?;
        
        // Initialize default loadout
        self.initialize_default_loadout()?;

        Ok(())
    }

    fn initialize_agent_rules(&self) -> Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agent_rules",
            [],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(());
        }

        let rules = vec![
            (
                "read_contributions_database",
                "When a user provides a SQLite contributions database file, you should:\n1. First, read the agent_rules table to understand how to use this database\n2. Read the repositories table to understand what repositories are tracked\n3. Read the contributions table to see what features/contributions have been documented\n4. Read the commits table for detailed commit information when needed\n5. Use the prompts table to find reusable prompts for common tasks\n6. Always check the updated_at timestamps to understand data freshness",
                10,
                "Database Usage",
            ),
            (
                "generate_contributions_markdown",
                "To generate or update a contributions markdown file:\n1. Query contributions table for the repository, ordered by priority DESC, then by name\n2. For each contribution, include: Name and overview, Key commits (look up details in commits table), Related commits, Technical details (from JSON field), Resume bullet points (from JSON array)\n3. Group related contributions by category\n4. Include timestamps from commits table for human-readable dates\n5. Always include author information from commits\n6. Maintain consistent formatting across all contribution files\n7. Update the markdown file, preserving existing structure where possible",
                9,
                "Documentation",
            ),
            (
                "maintain_consistency",
                "When working with contributions data:\n1. Always use the same structure and format for similar contributions\n2. Keep resume bullet points concise and action-oriented\n3. Technical details should include: technology_stack, patterns, integrations, storage, security\n4. Categories should be consistent: Core Feature, Integration, Infrastructure, Feature Enhancement, Feature, Configuration, Performance, Bug Fix\n5. Priority should reflect importance: 10 = critical/core, 9-8 = major features, 7-5 = important features, 4-1 = minor features/fixes\n6. When adding new contributions, follow existing patterns in the database",
                8,
                "Data Quality",
            ),
        ];

        for (name, instruction, priority, category) in rules {
            self.conn.execute(
                "INSERT INTO agent_rules (name, instruction, priority, category) VALUES (?1, ?2, ?3, ?4)",
                params![name, instruction, priority, category],
            )?;
        }

        Ok(())
    }

    fn initialize_prompts(&self) -> Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM prompts",
            [],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(());
        }

        let prompts = vec![
            (
                "analyze_contributions",
                "Analyze the contributions database for repository {repository_url}.\n\n1. Read all agent rules from the agent_rules table\n2. Query all contributions for this repository\n3. For each contribution, provide:\n   - Summary of what was built\n   - Key technical details\n   - Resume bullet points\n   - Associated commits with dates\n\nGenerate a comprehensive analysis following the patterns established in the database.",
                "Prompt for analyzing all contributions in a repository",
                "Analysis",
                r#"["repository_url"]"#,
            ),
            (
                "generate_contributions_markdown",
                "Update the contributions markdown file for repository {repository_url} based on the contributions database.\n\n1. Read the current markdown file if it exists\n2. Query contributions from database ordered by priority and category\n3. Generate/update markdown following the established format\n4. Include all contributions with their details\n5. Maintain consistency with existing documentation style\n6. Update timestamps and author information from commits table",
                "Prompt for updating contributions markdown file",
                "Documentation",
                r#"["repository_url"]"#,
            ),
        ];

        for (name, prompt_text, description, category, variables) in prompts {
            self.conn.execute(
                "INSERT INTO prompts (name, prompt_text, description, category, variables) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![name, prompt_text, description, category, variables],
            )?;
        }

        Ok(())
    }

    fn initialize_default_loadout(&self) -> Result<()> {
        // Check if default loadout exists
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM loadouts WHERE is_default = 1",
            [],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(()); // Default loadout already exists
        }

        // Create default loadout
        self.conn.execute(
            "INSERT INTO loadouts (name, is_default) VALUES ('default', 1)",
            [],
        )?;

        let loadout_id: i64 = self.conn.last_insert_rowid();

        // Associate all existing prompts with default loadout
        let mut stmt = self.conn.prepare("SELECT id FROM prompts")?;
        let prompt_rows = stmt.query_map([], |row| {
            row.get::<_, i64>(0)
        })?;

        for prompt_row in prompt_rows {
            let prompt_id = prompt_row?;
            self.conn.execute(
                "INSERT OR IGNORE INTO loadout_prompts (loadout_id, prompt_id) VALUES (?1, ?2)",
                params![loadout_id, prompt_id],
            )?;
        }

        // Associate all existing rules with default loadout
        let mut stmt = self.conn.prepare("SELECT id FROM agent_rules")?;
        let rule_rows = stmt.query_map([], |row| {
            row.get::<_, i64>(0)
        })?;

        for rule_row in rule_rows {
            let rule_id = rule_row?;
            self.conn.execute(
                "INSERT OR IGNORE INTO loadout_rules (loadout_id, rule_id) VALUES (?1, ?2)",
                params![loadout_id, rule_id],
            )?;
        }

        Ok(())
    }

    pub fn add_repository(&self, repo: &Repository) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO repositories (repository_url, organization, name, description, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![repo.url, repo.organization, repo.name, repo.description, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn add_contribution(&self, contrib: &Contribution) -> Result<i64> {
        let key_commits_json = serde_json::to_string(&contrib.key_commits)?;
        let related_commits_json = serde_json::to_string(&contrib.related_commits)?;
        let technical_details_json = serde_json::to_string(&contrib.technical_details)?;
        let resume_bullets_json = serde_json::to_string(&contrib.resume_bullets)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO contributions 
            (repository_url, name, overview, description, key_commits, related_commits, 
             technical_details, resume_bullets, category, priority, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                contrib.repository_url,
                contrib.name,
                contrib.overview,
                contrib.description,
                key_commits_json,
                related_commits_json,
                technical_details_json,
                resume_bullets_json,
                contrib.category,
                contrib.priority,
                Utc::now().to_rfc3339()
            ],
        )?;

        let id: i64 = if let Some(existing_id) = contrib.id {
            existing_id
        } else {
            self.conn.last_insert_rowid()
        };

        Ok(id)
    }

    pub fn add_commit(&self, commit: &Commit) -> Result<()> {
        let files_changed_json = serde_json::to_string(&commit.files_changed)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO commits 
            (commit_hash, repository_url, contribution_id, author, author_email, date, 
             message, files_changed, lines_added, lines_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                commit.hash,
                commit.repository_url,
                commit.contribution_id,
                commit.author,
                commit.author_email,
                commit.date,
                commit.message,
                files_changed_json,
                commit.lines_added,
                commit.lines_deleted
            ],
        )?;
        Ok(())
    }

    pub fn get_contributions(&self, repo_url: &str) -> Result<Vec<Contribution>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, repository_url, name, overview, description, key_commits, 
             related_commits, technical_details, resume_bullets, category, priority
             FROM contributions WHERE repository_url = ?1 ORDER BY priority DESC, name"
        )?;

        let rows = stmt.query_map(params![repo_url], |row| {
            Ok(Contribution {
                id: Some(row.get(0)?),
                repository_url: row.get(1)?,
                name: row.get(2)?,
                overview: row.get(3)?,
                description: row.get(4)?,
                key_commits: serde_json::from_str(row.get::<_, String>(5)?.as_str()).unwrap_or_default(),
                related_commits: serde_json::from_str(row.get::<_, String>(6)?.as_str()).unwrap_or_default(),
                technical_details: serde_json::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or_default(),
                resume_bullets: serde_json::from_str(row.get::<_, String>(8)?.as_str()).unwrap_or_default(),
                category: row.get(9)?,
                priority: row.get::<_, i32>(10)? as u8,
            })
        })?;

        let mut contributions = Vec::new();
        for row in rows {
            contributions.push(row?);
        }
        Ok(contributions)
    }

    pub fn get_contribution(&self, repo_url: &str, name: &str) -> Result<Option<Contribution>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, repository_url, name, overview, description, key_commits, 
             related_commits, technical_details, resume_bullets, category, priority
             FROM contributions WHERE repository_url = ?1 AND name = ?2"
        )?;

        let result = stmt.query_row(params![repo_url, name], |row| {
            Ok(Contribution {
                id: Some(row.get(0)?),
                repository_url: row.get(1)?,
                name: row.get(2)?,
                overview: row.get(3)?,
                description: row.get(4)?,
                key_commits: serde_json::from_str(row.get::<_, String>(5)?.as_str()).unwrap_or_default(),
                related_commits: serde_json::from_str(row.get::<_, String>(6)?.as_str()).unwrap_or_default(),
                technical_details: serde_json::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or_default(),
                resume_bullets: serde_json::from_str(row.get::<_, String>(8)?.as_str()).unwrap_or_default(),
                category: row.get(9)?,
                priority: row.get::<_, i32>(10)? as u8,
            })
        });

        match result {
            Ok(contrib) => Ok(Some(contrib)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_commits_for_contribution(&self, repo_url: &str, contrib_name: &str) -> Result<Vec<Commit>> {
        let mut stmt = self.conn.prepare(
            "SELECT cm.commit_hash, cm.repository_url, cm.contribution_id, cm.author, 
             cm.author_email, cm.date, cm.message, cm.files_changed, cm.lines_added, cm.lines_deleted
             FROM commits cm
             JOIN contributions c ON cm.contribution_id = c.id
             WHERE c.repository_url = ?1 AND c.name = ?2
             ORDER BY cm.date DESC"
        )?;

        let rows = stmt.query_map(params![repo_url, contrib_name], |row| {
            Ok(Commit {
                hash: row.get(0)?,
                repository_url: row.get(1)?,
                contribution_id: row.get(2)?,
                author: row.get(3)?,
                author_email: row.get(4)?,
                date: row.get(5)?,
                message: row.get(6)?,
                files_changed: serde_json::from_str(row.get::<_, String>(7)?.as_str()).unwrap_or_default(),
                lines_added: row.get(8)?,
                lines_deleted: row.get(9)?,
            })
        })?;

        let mut commits = Vec::new();
        for row in rows {
            commits.push(row?);
        }
        Ok(commits)
    }

    pub fn get_all_repositories(&self) -> Result<Vec<Repository>> {
        let mut stmt = self.conn.prepare(
            "SELECT repository_url, organization, name, description FROM repositories ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Repository {
                url: row.get(0)?,
                organization: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
            })
        })?;

        let mut repos = Vec::new();
        for row in rows {
            repos.push(row?);
        }
        Ok(repos)
    }

    #[allow(dead_code)]
    pub fn get_contribution_id(&self, repo_url: &str, name: &str) -> Result<Option<i64>> {
        let result: Result<i64, _> = self.conn.query_row(
            "SELECT id FROM contributions WHERE repository_url = ?1 AND name = ?2",
            params![repo_url, name],
            |row| row.get(0),
        );

        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_statistics(&self) -> Result<HashMap<String, i64>> {
        let mut stats = HashMap::new();

        stats.insert("repositories".to_string(), 
            self.conn.query_row("SELECT COUNT(*) FROM repositories", [], |row| row.get(0))?);
        stats.insert("contributions".to_string(), 
            self.conn.query_row("SELECT COUNT(*) FROM contributions", [], |row| row.get(0))?);
        stats.insert("commits".to_string(), 
            self.conn.query_row("SELECT COUNT(*) FROM commits", [], |row| row.get(0))?);
        stats.insert("agent_rules".to_string(), 
            self.conn.query_row("SELECT COUNT(*) FROM agent_rules", [], |row| row.get(0))?);
        stats.insert("prompts".to_string(), 
            self.conn.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))?);

        Ok(stats)
    }

    /// Get all unique organizations from repositories
    #[allow(dead_code)]
    pub fn get_all_organizations(&self) -> Result<Vec<(String, Option<String>)>> {
        let repos = self.get_all_repositories()?;
        let mut orgs: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
        
        for repo in repos {
            // Use organization name as key, description as value if available
            orgs.entry(repo.organization.clone())
                .or_insert_with(|| repo.description.clone());
        }
        
        Ok(orgs.into_iter().collect())
    }

    /// Load config from database (for syncing to TOML)
    pub fn load_config_from_db(&self) -> Result<crate::config::Config> {
        use crate::config::{Config, Organization, RepositoryConfig};
        use std::collections::HashMap;

        let mut config = Config::new();
        
        // Get all repositories
        let repos = self.get_all_repositories()?;
        
        // Build organizations map
        let mut orgs: HashMap<String, Option<String>> = HashMap::new();
        for repo in &repos {
            orgs.entry(repo.organization.clone())
                .or_insert_with(|| repo.description.clone());
        }
        
        // Convert to config format
        for (org_name, description) in orgs {
            config.organizations.insert(
                org_name.clone(),
                Organization {
                    name: org_name,
                    description,
                },
            );
        }
        
        // Add repositories
        for repo in repos {
            config.repositories.insert(
                repo.url.clone(),
                RepositoryConfig {
                    organization: repo.organization,
                    name: repo.name,
                    description: repo.description,
                },
            );
        }
        
        Ok(config)
    }

    /// Load config into database (for syncing from TOML)
    pub fn load_config_to_db(&self, config: &crate::config::Config) -> Result<()> {
        use crate::database::Repository;
        
        // Add organizations (as repositories with org info)
        // Note: Organizations are represented implicitly through repositories
        // We'll add repositories which will create the org structure
        
        // Add repositories
        for (url, repo_config) in &config.repositories {
            let repo = Repository {
                url: url.clone(),
                organization: repo_config.organization.clone(),
                name: repo_config.name.clone(),
                description: repo_config.description.clone(),
            };
            self.add_repository(&repo)?;
        }
        
        Ok(())
    }

    // Loadout management functions
    pub fn create_loadout(&self, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO loadouts (name, is_default) VALUES (?1, 0)",
            params![name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_loadout_id(&self, name: &str) -> Result<Option<i64>> {
        let result: Result<i64, _> = self.conn.query_row(
            "SELECT id FROM loadouts WHERE name = ?1",
            params![name],
            |row| row.get(0),
        );

        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_loadouts(&self) -> Result<Vec<(i64, String, bool)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, is_default FROM loadouts ORDER BY is_default DESC, name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, i32>(2)? != 0,
            ))
        })?;

        let mut loadouts = Vec::new();
        for row in rows {
            loadouts.push(row?);
        }
        Ok(loadouts)
    }

    pub fn delete_loadout(&self, name: &str) -> Result<()> {
        let loadout_id = self.get_loadout_id(name)?
            .ok_or_else(|| anyhow::anyhow!("Loadout '{}' not found", name))?;

        // Check if it's the default loadout
        let is_default: i32 = self.conn.query_row(
            "SELECT is_default FROM loadouts WHERE id = ?1",
            params![loadout_id],
            |row| row.get(0),
        )?;

        if is_default != 0 {
            return Err(anyhow::anyhow!("Cannot delete the default loadout"));
        }

        self.conn.execute(
            "DELETE FROM loadouts WHERE id = ?1",
            params![loadout_id],
        )?;
        Ok(())
    }

    pub fn save_current_to_loadout(&self, loadout_name: &str) -> Result<()> {
        let loadout_id = self.get_loadout_id(loadout_name)?
            .ok_or_else(|| anyhow::anyhow!("Loadout '{}' not found", loadout_name))?;

        // Clear existing associations
        self.conn.execute(
            "DELETE FROM loadout_prompts WHERE loadout_id = ?1",
            params![loadout_id],
        )?;
        self.conn.execute(
            "DELETE FROM loadout_rules WHERE loadout_id = ?1",
            params![loadout_id],
        )?;

        // Add all current prompts
        let mut stmt = self.conn.prepare("SELECT id FROM prompts")?;
        let prompt_rows = stmt.query_map([], |row| {
            row.get::<_, i64>(0)
        })?;

        for prompt_row in prompt_rows {
            let prompt_id = prompt_row?;
            self.conn.execute(
                "INSERT INTO loadout_prompts (loadout_id, prompt_id) VALUES (?1, ?2)",
                params![loadout_id, prompt_id],
            )?;
        }

        // Add all current rules
        let mut stmt = self.conn.prepare("SELECT id FROM agent_rules")?;
        let rule_rows = stmt.query_map([], |row| {
            row.get::<_, i64>(0)
        })?;

        for rule_row in rule_rows {
            let rule_id = rule_row?;
            self.conn.execute(
                "INSERT INTO loadout_rules (loadout_id, rule_id) VALUES (?1, ?2)",
                params![loadout_id, rule_id],
            )?;
        }

        Ok(())
    }

    pub fn load_loadout(&self, loadout_name: &str) -> Result<()> {
        let loadout_id = self.get_loadout_id(loadout_name)?
            .ok_or_else(|| anyhow::anyhow!("Loadout '{}' not found", loadout_name))?;

        // Get prompts from loadout
        let mut stmt = self.conn.prepare(
            "SELECT prompt_id FROM loadout_prompts WHERE loadout_id = ?1"
        )?;
        let prompt_rows = stmt.query_map(params![loadout_id], |row| {
            row.get::<_, i64>(0)
        })?;

        let mut loadout_prompt_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        for prompt_row in prompt_rows {
            loadout_prompt_ids.insert(prompt_row?);
        }

        // Get rules from loadout
        let mut stmt = self.conn.prepare(
            "SELECT rule_id FROM loadout_rules WHERE loadout_id = ?1"
        )?;
        let rule_rows = stmt.query_map(params![loadout_id], |row| {
            row.get::<_, i64>(0)
        })?;

        let mut loadout_rule_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        for rule_row in rule_rows {
            loadout_rule_ids.insert(rule_row?);
        }

        // Delete prompts not in loadout
        let all_prompts: Vec<i64> = {
            let mut stmt = self.conn.prepare("SELECT id FROM prompts")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        for prompt_id in all_prompts {
            if !loadout_prompt_ids.contains(&prompt_id) {
                self.conn.execute(
                    "DELETE FROM prompts WHERE id = ?1",
                    params![prompt_id],
                )?;
            }
        }

        // Delete rules not in loadout
        let all_rules: Vec<i64> = {
            let mut stmt = self.conn.prepare("SELECT id FROM agent_rules")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        for rule_id in all_rules {
            if !loadout_rule_ids.contains(&rule_id) {
                self.conn.execute(
                    "DELETE FROM agent_rules WHERE id = ?1",
                    params![rule_id],
                )?;
            }
        }

        Ok(())
    }

    pub fn reload_default_loadout(&self) -> Result<()> {
        self.load_loadout("default")
    }

    pub fn get_all_agent_rules(&self) -> Result<Vec<AgentRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, instruction, priority, category FROM agent_rules ORDER BY priority DESC, name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    pub fn get_all_prompts(&self) -> Result<Vec<PromptInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, prompt_text, description, category FROM prompts ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row?);
        }
        Ok(prompts)
    }
}

