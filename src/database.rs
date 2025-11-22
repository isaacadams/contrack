use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json;
use std::collections::HashMap;

use crate::utils::get_database_path;

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
}

