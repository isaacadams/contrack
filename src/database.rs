use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

use crate::git::GitCommit;
use crate::utils::get_database_path;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackedRepository {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub local_path: String,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Contribution {
    pub id: i64,
    pub repository_id: i64,
    pub repository_slug: String,
    pub name: String,
    pub overview: String,
    pub description: String,
    pub category: String,
    pub priority: u8,
    pub status: String,
    pub confidence: Option<String>,
    pub rationale: Option<String>,
    pub covered_prs: Vec<i64>,
    pub key_commit_refs: Vec<String>,
    pub related_commit_refs: Vec<String>,
    pub technical_details: Vec<String>,
    pub resume_bullets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredCommit {
    pub hash: String,
    pub repository_id: i64,
    pub repository_slug: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub committed_at: String,
    pub summary: String,
    pub body: Option<String>,
    pub files_changed: Vec<String>,
    pub lines_added: i64,
    pub lines_deleted: i64,
}

#[derive(Debug, Clone)]
pub struct NewRepository {
    pub slug: String,
    pub name: String,
    pub local_path: String,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewContribution {
    pub repository_id: i64,
    pub name: String,
    pub overview: String,
    pub description: String,
    pub category: String,
    pub priority: u8,
    pub status: String,
    pub confidence: Option<String>,
    pub rationale: Option<String>,
    pub covered_prs: Vec<i64>,
    pub key_commit_refs: Vec<String>,
    pub related_commit_refs: Vec<String>,
    pub technical_details: Vec<String>,
    pub resume_bullets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStats {
    pub repositories: usize,
    pub contributions: usize,
    pub commits: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitAuthorSummary {
    pub author_name: String,
    pub author_email: Option<String>,
    pub commit_count: usize,
    pub latest_commit_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryStatus {
    pub repository: TrackedRepository,
    pub contributions: usize,
    pub commits: usize,
    pub latest_commit_at: Option<String>,
    pub latest_imported_at: Option<String>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let path = get_database_path()?;
        Self::open_at(&path)
    }

    pub fn open_at(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {}", path.display()))?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        let database = Self { conn };
        database.initialize_schema()?;
        Ok(database)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                local_path TEXT NOT NULL UNIQUE,
                remote_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contributions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repository_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                overview TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                priority INTEGER NOT NULL,
                key_commit_refs TEXT NOT NULL DEFAULT '[]',
                related_commit_refs TEXT NOT NULL DEFAULT '[]',
                technical_details TEXT NOT NULL DEFAULT '[]',
                resume_bullets TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
                UNIQUE (repository_id, name)
            );

            CREATE TABLE IF NOT EXISTS commits (
                hash TEXT PRIMARY KEY,
                repository_id INTEGER NOT NULL,
                author_name TEXT NOT NULL,
                author_email TEXT,
                committed_at TEXT NOT NULL,
                summary TEXT NOT NULL,
                body TEXT,
                files_changed TEXT NOT NULL DEFAULT '[]',
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_deleted INTEGER NOT NULL DEFAULT 0,
                imported_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_contributions_repository_id ON contributions(repository_id);
            CREATE INDEX IF NOT EXISTS idx_commits_repository_id ON commits(repository_id);
            CREATE INDEX IF NOT EXISTS idx_commits_committed_at ON commits(committed_at DESC);
            "
        )?;

        self.ensure_column("contributions", "status", "TEXT NOT NULL DEFAULT 'draft'")?;
        self.ensure_column("contributions", "confidence", "TEXT")?;
        self.ensure_column("contributions", "rationale", "TEXT")?;
        self.ensure_column("contributions", "covered_prs", "TEXT NOT NULL DEFAULT '[]'")?;

        Ok(())
    }

    fn ensure_column(&self, table: &str, column: &str, definition: &str) -> Result<()> {
        if self.column_exists(table, column)? {
            return Ok(());
        }

        self.conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )?;
        Ok(())
    }

    fn column_exists(&self, table: &str, column: &str) -> Result<bool> {
        let pragma = format!("PRAGMA table_info({table})");
        let mut statement = self.conn.prepare(&pragma)?;
        let mut rows = statement.query([])?;

        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == column {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn upsert_repository(&self, repository: &NewRepository) -> Result<TrackedRepository> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "
            INSERT INTO repositories (slug, name, local_path, remote_url, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?5)
            ON CONFLICT(local_path) DO UPDATE SET
                slug = excluded.slug,
                name = excluded.name,
                remote_url = excluded.remote_url,
                updated_at = excluded.updated_at
            ",
            params![
                repository.slug,
                repository.name,
                repository.local_path,
                repository.remote_url,
                now
            ],
        )?;

        self.get_repository(&repository.slug)?
            .context("Repository could not be loaded after save")
    }

    pub fn list_repositories(&self) -> Result<Vec<TrackedRepository>> {
        let mut statement = self.conn.prepare(
            "SELECT id, slug, name, local_path, remote_url FROM repositories ORDER BY slug ASC",
        )?;
        let rows = statement.query_map([], map_repository_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_repository(&self, selector: &str) -> Result<Option<TrackedRepository>> {
        let normalized = selector.trim();
        let mut statement = self.conn.prepare(
            "
            SELECT id, slug, name, local_path, remote_url
            FROM repositories
            WHERE slug = ?1 OR name = ?1 OR local_path = ?1 OR remote_url = ?1
            ORDER BY slug ASC
            ",
        )?;
        let matches = statement
            .query_map([normalized], map_repository_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(anyhow!(
                "Repository selector '{normalized}' is ambiguous. Use the repository slug shown in `contrack repo list`."
            )),
        }
    }

    pub fn remove_repository(&self, repository_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM repositories WHERE id = ?1", [repository_id])?;
        Ok(())
    }

    pub fn add_contribution(&self, contribution: &NewContribution) -> Result<Contribution> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "
            INSERT INTO contributions (
                repository_id, name, overview, description, category, priority, status, confidence,
                rationale, covered_prs, key_commit_refs, related_commit_refs, technical_details,
                resume_bullets, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)
            ",
            params![
                contribution.repository_id,
                contribution.name,
                contribution.overview,
                contribution.description,
                contribution.category,
                contribution.priority,
                contribution.status,
                contribution.confidence,
                contribution.rationale,
                to_json(&contribution.covered_prs)?,
                to_json(&contribution.key_commit_refs)?,
                to_json(&contribution.related_commit_refs)?,
                to_json(&contribution.technical_details)?,
                to_json(&contribution.resume_bullets)?,
                now,
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_contribution_by_id(id)?
            .context("Contribution could not be loaded after save")
    }

    pub fn update_contribution(&self, contribution: &Contribution) -> Result<()> {
        self.conn.execute(
            "
            UPDATE contributions
            SET name = ?2, overview = ?3, description = ?4, category = ?5, priority = ?6,
                status = ?7, confidence = ?8, rationale = ?9, covered_prs = ?10,
                key_commit_refs = ?11, related_commit_refs = ?12, technical_details = ?13,
                resume_bullets = ?14, updated_at = ?15
            WHERE id = ?1
            ",
            params![
                contribution.id,
                contribution.name,
                contribution.overview,
                contribution.description,
                contribution.category,
                contribution.priority,
                contribution.status,
                contribution.confidence,
                contribution.rationale,
                to_json(&contribution.covered_prs)?,
                to_json(&contribution.key_commit_refs)?,
                to_json(&contribution.related_commit_refs)?,
                to_json(&contribution.technical_details)?,
                to_json(&contribution.resume_bullets)?,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_contribution(&self, contribution_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM contributions WHERE id = ?1", [contribution_id])?;
        Ok(())
    }

    pub fn list_contributions(
        &self,
        repository_id: Option<i64>,
        status: Option<&str>,
    ) -> Result<Vec<Contribution>> {
        let sql = "
            SELECT
                c.id, c.repository_id, r.slug, c.name, c.overview, c.description, c.category,
                c.priority, c.status, c.confidence, c.rationale, c.covered_prs,
                c.key_commit_refs, c.related_commit_refs, c.technical_details, c.resume_bullets
            FROM contributions c
            JOIN repositories r ON r.id = c.repository_id
            WHERE (?1 IS NULL OR c.repository_id = ?1)
              AND (?2 IS NULL OR c.status = ?2)
            ORDER BY c.priority DESC, c.category ASC, c.name ASC
        ";
        let mut statement = self.conn.prepare(sql)?;
        let rows = statement.query_map(params![repository_id, status], map_contribution_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_contribution_by_id(&self, id: i64) -> Result<Option<Contribution>> {
        let sql = "
            SELECT
                c.id, c.repository_id, r.slug, c.name, c.overview, c.description, c.category,
                c.priority, c.status, c.confidence, c.rationale, c.covered_prs,
                c.key_commit_refs, c.related_commit_refs, c.technical_details, c.resume_bullets
            FROM contributions c
            JOIN repositories r ON r.id = c.repository_id
            WHERE c.id = ?1
        ";
        self.conn
            .query_row(sql, [id], map_contribution_row)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_contribution(&self, selector: &str) -> Result<Option<Contribution>> {
        if let Ok(id) = selector.parse::<i64>() {
            return self.get_contribution_by_id(id);
        }

        let sql = "
            SELECT
                c.id, c.repository_id, r.slug, c.name, c.overview, c.description, c.category,
                c.priority, c.status, c.confidence, c.rationale, c.covered_prs,
                c.key_commit_refs, c.related_commit_refs, c.technical_details, c.resume_bullets
            FROM contributions c
            JOIN repositories r ON r.id = c.repository_id
            WHERE c.name = ?1
            ORDER BY c.priority DESC, c.name ASC
        ";
        let mut statement = self.conn.prepare(sql)?;
        let matches = statement
            .query_map([selector], map_contribution_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(anyhow!(
                "Contribution selector '{selector}' is ambiguous. Use the numeric contribution id from `contrack contribution list`."
            )),
        }
    }

    pub fn import_commits(&self, repository_id: i64, commits: &[GitCommit]) -> Result<usize> {
        let imported_at = Utc::now().to_rfc3339();
        let mut inserted = 0_usize;

        for commit in commits {
            inserted += self.conn.execute(
                "
                INSERT INTO commits (
                    hash, repository_id, author_name, author_email, committed_at, summary, body,
                    files_changed, lines_added, lines_deleted, imported_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(hash) DO UPDATE SET
                    repository_id = excluded.repository_id,
                    author_name = excluded.author_name,
                    author_email = excluded.author_email,
                    committed_at = excluded.committed_at,
                    summary = excluded.summary,
                    body = excluded.body,
                    files_changed = excluded.files_changed,
                    lines_added = excluded.lines_added,
                    lines_deleted = excluded.lines_deleted,
                    imported_at = excluded.imported_at
                ",
                params![
                    commit.hash,
                    repository_id,
                    commit.author_name,
                    commit.author_email,
                    commit.committed_at,
                    commit.summary,
                    commit.body,
                    to_json(&commit.files_changed)?,
                    commit.lines_added,
                    commit.lines_deleted,
                    imported_at,
                ],
            )?;
        }

        Ok(inserted)
    }

    pub fn list_commits(
        &self,
        repository_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<StoredCommit>> {
        let sql = "
            SELECT c.hash, c.repository_id, r.slug, c.author_name, c.author_email, c.committed_at,
                   c.summary, c.body, c.files_changed, c.lines_added, c.lines_deleted
            FROM commits c
            JOIN repositories r ON r.id = c.repository_id
            WHERE (?1 IS NULL OR c.repository_id = ?1)
            ORDER BY c.committed_at DESC, c.hash DESC
            LIMIT ?2
        ";
        let mut statement = self.conn.prepare(sql)?;
        let rows = statement.query_map(params![repository_id, limit as i64], map_commit_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn list_all_commits_for_repository(&self, repository_id: i64) -> Result<Vec<StoredCommit>> {
        let sql = "
            SELECT c.hash, c.repository_id, r.slug, c.author_name, c.author_email, c.committed_at,
                   c.summary, c.body, c.files_changed, c.lines_added, c.lines_deleted
            FROM commits c
            JOIN repositories r ON r.id = c.repository_id
            WHERE c.repository_id = ?1
            ORDER BY c.committed_at DESC, c.hash DESC
        ";
        let mut statement = self.conn.prepare(sql)?;
        let rows = statement.query_map([repository_id], map_commit_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn stats(&self, repository_id: Option<i64>) -> Result<DatabaseStats> {
        let repositories = if repository_id.is_some() {
            1
        } else {
            count_to_usize(self.conn.query_row(
                "SELECT COUNT(*) FROM repositories",
                [],
                |row| row.get::<_, i64>(0),
            )?)?
        };
        let contributions = count_to_usize(self.conn.query_row(
            "SELECT COUNT(*) FROM contributions WHERE (?1 IS NULL OR repository_id = ?1)",
            [repository_id],
            |row| row.get::<_, i64>(0),
        )?)?;
        let commits = count_to_usize(self.conn.query_row(
            "SELECT COUNT(*) FROM commits WHERE (?1 IS NULL OR repository_id = ?1)",
            [repository_id],
            |row| row.get::<_, i64>(0),
        )?)?;
        Ok(DatabaseStats {
            repositories,
            contributions,
            commits,
        })
    }

    pub fn list_commit_authors(
        &self,
        repository_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<CommitAuthorSummary>> {
        let sql = "
            SELECT c.author_name, c.author_email, COUNT(*) AS commit_count, MAX(c.committed_at) AS latest_commit_at
            FROM commits c
            WHERE (?1 IS NULL OR c.repository_id = ?1)
            GROUP BY c.author_name, c.author_email
            ORDER BY commit_count DESC, latest_commit_at DESC, c.author_name ASC
            LIMIT ?2
        ";
        let mut statement = self.conn.prepare(sql)?;
        let rows = statement.query_map(params![repository_id, limit as i64], |row| {
            Ok(CommitAuthorSummary {
                author_name: row.get(0)?,
                author_email: row.get(1)?,
                commit_count: count_to_usize(row.get::<_, i64>(2)?).map_err(sql_conv_err)?,
                latest_commit_at: row.get(3)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn list_repository_statuses(
        &self,
        repository_id: Option<i64>,
    ) -> Result<Vec<RepositoryStatus>> {
        let sql = "
            SELECT
                r.id,
                r.slug,
                r.name,
                r.local_path,
                r.remote_url,
                COUNT(DISTINCT c.id) AS contribution_count,
                COUNT(DISTINCT m.hash) AS commit_count,
                MAX(m.committed_at) AS latest_commit_at,
                MAX(m.imported_at) AS latest_imported_at
            FROM repositories r
            LEFT JOIN contributions c ON c.repository_id = r.id
            LEFT JOIN commits m ON m.repository_id = r.id
            WHERE (?1 IS NULL OR r.id = ?1)
            GROUP BY r.id, r.slug, r.name, r.local_path, r.remote_url
            ORDER BY r.slug ASC
        ";
        let mut statement = self.conn.prepare(sql)?;
        let rows = statement.query_map(params![repository_id], |row| {
            Ok(RepositoryStatus {
                repository: TrackedRepository {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    local_path: row.get(3)?,
                    remote_url: row.get(4)?,
                },
                contributions: count_to_usize(row.get::<_, i64>(5)?).map_err(sql_conv_err)?,
                commits: count_to_usize(row.get::<_, i64>(6)?).map_err(sql_conv_err)?,
                latest_commit_at: row.get(7)?,
                latest_imported_at: row.get(8)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }
}

fn count_to_usize(count: i64) -> Result<usize> {
    usize::try_from(count).context("Database count exceeded supported size")
}

fn map_repository_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackedRepository> {
    Ok(TrackedRepository {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        local_path: row.get(3)?,
        remote_url: row.get(4)?,
    })
}

fn map_contribution_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contribution> {
    Ok(Contribution {
        id: row.get(0)?,
        repository_id: row.get(1)?,
        repository_slug: row.get(2)?,
        name: row.get(3)?,
        overview: row.get(4)?,
        description: row.get(5)?,
        category: row.get(6)?,
        priority: row.get(7)?,
        status: row.get(8)?,
        confidence: row.get(9)?,
        rationale: row.get(10)?,
        covered_prs: from_json(&row.get::<_, String>(11)?).map_err(json_err)?,
        key_commit_refs: from_json(&row.get::<_, String>(12)?).map_err(json_err)?,
        related_commit_refs: from_json(&row.get::<_, String>(13)?).map_err(json_err)?,
        technical_details: from_json(&row.get::<_, String>(14)?).map_err(json_err)?,
        resume_bullets: from_json(&row.get::<_, String>(15)?).map_err(json_err)?,
    })
}

fn map_commit_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredCommit> {
    Ok(StoredCommit {
        hash: row.get(0)?,
        repository_id: row.get(1)?,
        repository_slug: row.get(2)?,
        author_name: row.get(3)?,
        author_email: row.get(4)?,
        committed_at: row.get(5)?,
        summary: row.get(6)?,
        body: row.get(7)?,
        files_changed: from_json(&row.get::<_, String>(8)?).map_err(json_err)?,
        lines_added: row.get(9)?,
        lines_deleted: row.get(10)?,
    })
}

fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).context("Failed to serialize JSON data")
}

fn from_json<T: serde::de::DeserializeOwned>(raw: &str) -> serde_json::Result<T> {
    serde_json::from_str(raw)
}

fn json_err(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn sql_conv_err(error: anyhow::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Integer,
        Box::<dyn std::error::Error + Send + Sync>::from(error),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn migrates_and_saves_rich_contributions() {
        let temp = TempDir::new().expect("tempdir");
        let db = Database::open_at(&temp.path().join("test.db")).expect("db");

        let repository = db
            .upsert_repository(&NewRepository {
                slug: "contrack".to_string(),
                name: "Contrack".to_string(),
                local_path: "/tmp/contrack".to_string(),
                remote_url: Some("https://github.com/isaacadams/contrack".to_string()),
            })
            .expect("repo");

        let contribution = db
            .add_contribution(&NewContribution {
                repository_id: repository.id,
                name: "Markdown generator".to_string(),
                overview: "Built polished Markdown output.".to_string(),
                description: "Created grouped output for resume and portfolio use.".to_string(),
                category: "Feature".to_string(),
                priority: 5,
                status: "draft".to_string(),
                confidence: Some("high".to_string()),
                rationale: Some("Representative feature work".to_string()),
                covered_prs: vec![12, 15],
                key_commit_refs: vec!["abc123".to_string()],
                related_commit_refs: vec!["def456".to_string()],
                technical_details: vec!["Uses grouped category sections".to_string()],
                resume_bullets: vec!["Turned git history into resume-ready bullets".to_string()],
            })
            .expect("contribution");

        let loaded = db
            .get_contribution_by_id(contribution.id)
            .expect("query")
            .expect("existing contribution");

        assert_eq!(loaded.covered_prs, vec![12, 15]);
        assert_eq!(loaded.confidence.as_deref(), Some("high"));
        assert_eq!(loaded.status, "draft");
    }

    #[test]
    fn upgrades_older_contribution_schema() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("legacy.db");
        let conn = Connection::open(&path).expect("legacy db");
        conn.execute_batch(
            "
            CREATE TABLE repositories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                local_path TEXT NOT NULL UNIQUE,
                remote_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contributions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repository_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                overview TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                priority INTEGER NOT NULL,
                key_commit_refs TEXT NOT NULL DEFAULT '[]',
                related_commit_refs TEXT NOT NULL DEFAULT '[]',
                technical_details TEXT NOT NULL DEFAULT '[]',
                resume_bullets TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )
        .expect("legacy schema");

        drop(conn);

        let db = Database::open_at(&path).expect("migrated db");
        let repository = db
            .upsert_repository(&NewRepository {
                slug: "legacy".to_string(),
                name: "Legacy".to_string(),
                local_path: "/tmp/legacy".to_string(),
                remote_url: None,
            })
            .expect("repo");
        let contribution = db
            .add_contribution(&NewContribution {
                repository_id: repository.id,
                name: "Migrated contribution".to_string(),
                overview: "Overview".to_string(),
                description: "Description".to_string(),
                category: "Feature".to_string(),
                priority: 3,
                status: "accepted".to_string(),
                confidence: Some("medium".to_string()),
                rationale: Some("migration test".to_string()),
                covered_prs: vec![44],
                key_commit_refs: vec!["abc123".to_string()],
                related_commit_refs: Vec::new(),
                technical_details: Vec::new(),
                resume_bullets: Vec::new(),
            })
            .expect("contribution");

        assert_eq!(contribution.status, "accepted");
        assert_eq!(contribution.covered_prs, vec![44]);
    }

    #[test]
    fn lists_commit_authors_and_repository_status() {
        let temp = TempDir::new().expect("tempdir");
        let db = Database::open_at(&temp.path().join("test.db")).expect("db");

        let repository = db
            .upsert_repository(&NewRepository {
                slug: "contrack".to_string(),
                name: "Contrack".to_string(),
                local_path: "/tmp/contrack".to_string(),
                remote_url: Some("https://github.com/isaacadams/contrack".to_string()),
            })
            .expect("repo");

        db.import_commits(
            repository.id,
            &[
                GitCommit {
                    hash: "abc123".to_string(),
                    author_name: "Isaac Adams".to_string(),
                    author_email: Some("isaac@example.com".to_string()),
                    committed_at: "2026-04-08T00:00:00Z".to_string(),
                    summary: "first".to_string(),
                    body: None,
                    files_changed: vec!["src/main.rs".to_string()],
                    lines_added: 10,
                    lines_deleted: 1,
                },
                GitCommit {
                    hash: "def456".to_string(),
                    author_name: "Isaac Adams".to_string(),
                    author_email: Some("isaac@example.com".to_string()),
                    committed_at: "2026-04-09T00:00:00Z".to_string(),
                    summary: "second".to_string(),
                    body: None,
                    files_changed: vec!["src/commands.rs".to_string()],
                    lines_added: 7,
                    lines_deleted: 2,
                },
                GitCommit {
                    hash: "789abc".to_string(),
                    author_name: "Other Dev".to_string(),
                    author_email: Some("other@example.com".to_string()),
                    committed_at: "2026-04-07T00:00:00Z".to_string(),
                    summary: "third".to_string(),
                    body: None,
                    files_changed: vec!["README.md".to_string()],
                    lines_added: 3,
                    lines_deleted: 0,
                },
            ],
        )
        .expect("commits");

        db.add_contribution(&NewContribution {
            repository_id: repository.id,
            name: "Author stats".to_string(),
            overview: "Overview".to_string(),
            description: "Description".to_string(),
            category: "Tooling".to_string(),
            priority: 3,
            status: "draft".to_string(),
            confidence: None,
            rationale: None,
            covered_prs: vec![],
            key_commit_refs: vec!["abc123".to_string()],
            related_commit_refs: vec![],
            technical_details: vec![],
            resume_bullets: vec![],
        })
        .expect("contribution");

        let authors = db
            .list_commit_authors(Some(repository.id), 10)
            .expect("authors");
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].author_name, "Isaac Adams");
        assert_eq!(authors[0].commit_count, 2);
        assert_eq!(authors[0].latest_commit_at, "2026-04-09T00:00:00Z");

        let statuses = db
            .list_repository_statuses(Some(repository.id))
            .expect("statuses");
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].repository.slug, "contrack");
        assert_eq!(statuses[0].contributions, 1);
        assert_eq!(statuses[0].commits, 3);
        assert_eq!(
            statuses[0].latest_commit_at.as_deref(),
            Some("2026-04-09T00:00:00Z")
        );
        assert!(statuses[0].latest_imported_at.is_some());
    }
}
