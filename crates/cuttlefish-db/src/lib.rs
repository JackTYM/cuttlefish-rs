#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Database layer for Cuttlefish using SQLite + sqlx.

/// Database model types for all tables.
pub mod models;

use sqlx::{Row, SqlitePool};
use std::path::Path;

/// Database connection wrapper.
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open or create a database at the given path, run migrations.
    pub async fn open(path: &Path) -> Result<Self, sqlx::Error> {
        let path_str = path.to_string_lossy();
        let url = format!("sqlite://{}?mode=rwc", path_str);

        let pool = SqlitePool::connect(&url).await?;

        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    template_name TEXT,
    github_url TEXT,
    discord_channel_id TEXT,
    discord_guild_id TEXT,
    docker_container_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    model_used TEXT,
    token_count INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS agent_sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    agent_role TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    content_md TEXT NOT NULL,
    language TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS build_logs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'running',
    command TEXT NOT NULL,
    output TEXT NOT NULL DEFAULT '',
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS config_overrides (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(project_id, key)
)"#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_conversations_project_created ON conversations(project_id, created_at DESC)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_conversations_archived ON conversations(project_id, archived, created_at DESC)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_projects_discord_channel ON projects(discord_channel_id)",
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status)")
            .execute(pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_agent_sessions_project ON agent_sessions(project_id, started_at DESC)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_build_logs_project ON build_logs(project_id, started_at DESC)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get a reference to the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create a new project. Returns the created project.
    pub async fn create_project(
        &self,
        id: &str,
        name: &str,
        description: &str,
        template_name: Option<&str>,
    ) -> Result<models::Project, sqlx::Error> {
        sqlx::query_as::<_, models::Project>(
            r#"INSERT INTO projects (id, name, description, template_name) VALUES (?, ?, ?, ?) RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(template_name)
        .fetch_one(&self.pool)
        .await
    }

    /// Get a project by ID.
    pub async fn get_project(&self, id: &str) -> Result<Option<models::Project>, sqlx::Error> {
        sqlx::query_as::<_, models::Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// List all active projects.
    pub async fn list_active_projects(&self) -> Result<Vec<models::Project>, sqlx::Error> {
        sqlx::query_as::<_, models::Project>(
            "SELECT * FROM projects WHERE status = 'active' ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Update project status.
    pub async fn update_project_status(&self, id: &str, status: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Insert a conversation message.
    pub async fn insert_message(
        &self,
        id: &str,
        project_id: &str,
        role: &str,
        content: &str,
        model_used: Option<&str>,
        token_count: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO conversations (id, project_id, role, content, model_used, token_count) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(project_id)
        .bind(role)
        .bind(content)
        .bind(model_used)
        .bind(token_count)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get recent messages for a project (sliding window).
    pub async fn get_recent_messages(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<models::Conversation>, sqlx::Error> {
        sqlx::query_as::<_, models::Conversation>(
            "SELECT * FROM conversations WHERE project_id = ? AND archived = 0 ORDER BY created_at DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get total token count for a project.
    pub async fn get_total_token_count(&self, project_id: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COALESCE(SUM(token_count), 0) as total FROM conversations WHERE project_id = ? AND archived = 0")
            .bind(project_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("total"))
    }

    /// Get recent messages for a project in chronological order (oldest first).
    ///
    /// Returns the most recent `limit` messages, sorted oldest-first for use as
    /// conversation context. Excludes archived messages.
    pub async fn get_recent_messages_chrono(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<models::Conversation>, sqlx::Error> {
        // Subquery: get the N most recent, then re-sort oldest-first
        sqlx::query_as::<_, models::Conversation>(
            r#"SELECT * FROM (
                SELECT * FROM conversations
                WHERE project_id = ? AND archived = 0
                ORDER BY created_at DESC
                LIMIT ?
            ) sub ORDER BY created_at ASC"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get total count of non-archived messages for a project.
    pub async fn get_message_count(&self, project_id: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM conversations WHERE project_id = ? AND archived = 0",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get::<i64, _>("count"))
    }

    /// Archive messages older than a given `created_at` timestamp and insert a summary.
    ///
    /// This implements the sliding window summarization: older messages are archived
    /// (marked as `archived = 1`) and replaced with a summary system message.
    ///
    /// Returns the number of messages archived.
    pub async fn archive_and_summarize(
        &self,
        project_id: &str,
        before_created_at: &str,
        summary_id: &str,
        summary_content: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE conversations SET archived = 1 WHERE project_id = ? AND created_at < ? AND archived = 0",
        )
        .bind(project_id)
        .bind(before_created_at)
        .execute(&self.pool)
        .await?;

        let archived_count = result.rows_affected();

        sqlx::query(
            "INSERT INTO conversations (id, project_id, role, content, model_used, token_count, created_at) VALUES (?, ?, 'system', ?, 'summarizer', 0, ?)",
        )
        .bind(summary_id)
        .bind(project_id)
        .bind(summary_content)
        .bind(before_created_at)
        .execute(&self.pool)
        .await?;

        Ok(archived_count)
    }

    /// Get the creation timestamp of the Nth most recent non-archived message.
    ///
    /// Used to determine the cutoff point for sliding window summarization.
    /// Returns `None` if fewer than `n` messages exist.
    pub async fn get_nth_recent_message_timestamp(
        &self,
        project_id: &str,
        n: i64,
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT created_at FROM conversations WHERE project_id = ? AND archived = 0 ORDER BY created_at DESC LIMIT 1 OFFSET ?",
        )
        .bind(project_id)
        .bind(n)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.get::<String, _>("created_at")))
    }

    /// Get a project by its Discord channel ID.
    pub async fn get_project_by_discord_channel(
        &self,
        channel_id: &str,
    ) -> Result<Option<models::Project>, sqlx::Error> {
        sqlx::query_as::<_, models::Project>(
            "SELECT * FROM projects WHERE discord_channel_id = ?",
        )
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Get all projects for a specific Discord guild.
    pub async fn get_projects_by_guild(
        &self,
        guild_id: &str,
    ) -> Result<Vec<models::Project>, sqlx::Error> {
        sqlx::query_as::<_, models::Project>(
            "SELECT * FROM projects WHERE discord_guild_id = ? ORDER BY created_at DESC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Update the Discord channel and guild IDs for a project.
    pub async fn set_project_discord_channel(
        &self,
        project_id: &str,
        channel_id: &str,
        guild_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE projects SET discord_channel_id = ?, discord_guild_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(channel_id)
        .bind(guild_id)
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update the Docker container ID for a project.
    pub async fn set_project_container(
        &self,
        project_id: &str,
        container_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE projects SET docker_container_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(container_id)
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update the GitHub URL for a project.
    pub async fn set_project_github_url(
        &self,
        project_id: &str,
        github_url: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE projects SET github_url = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(github_url)
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert a template into the database.
    pub async fn create_template(
        &self,
        id: &str,
        name: &str,
        description: &str,
        content_md: &str,
        language: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO templates (id, name, description, content_md, language) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(content_md)
        .bind(language)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get a template by name.
    pub async fn get_template(
        &self,
        name: &str,
    ) -> Result<Option<models::Template>, sqlx::Error> {
        sqlx::query_as::<_, models::Template>("SELECT * FROM templates WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

    /// List all templates, optionally filtered by language.
    pub async fn list_templates(
        &self,
        language: Option<&str>,
    ) -> Result<Vec<models::Template>, sqlx::Error> {
        if let Some(lang) = language {
            sqlx::query_as::<_, models::Template>(
                "SELECT * FROM templates WHERE language = ? ORDER BY name ASC",
            )
            .bind(lang)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, models::Template>("SELECT * FROM templates ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await
        }
    }

    /// Delete a template by name.
    pub async fn delete_template(&self, name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM templates WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_db() -> (Database, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).await.expect("db open");
        (db, dir)
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let (db, _dir) = test_db().await;
        let id = uuid::Uuid::new_v4().to_string();
        db.create_project(&id, "test-project", "A test project", None)
            .await
            .expect("create");
        let project = db
            .get_project(&id)
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(project.name, "test-project");
        assert_eq!(project.status, "active");
    }

    #[tokio::test]
    async fn test_update_project_status() {
        let (db, _dir) = test_db().await;
        let id = uuid::Uuid::new_v4().to_string();
        db.create_project(&id, "test2", "Desc", None)
            .await
            .expect("create");
        db.update_project_status(&id, "completed")
            .await
            .expect("update");
        let project = db
            .get_project(&id)
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(project.status, "completed");
    }

    #[tokio::test]
    async fn test_insert_and_get_messages() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "test3", "Desc", None)
            .await
            .expect("create");

        for i in 0..5 {
            let msg_id = uuid::Uuid::new_v4().to_string();
            db.insert_message(&msg_id, &project_id, "user", &format!("message {}", i), None, 10)
                .await
                .expect("insert");
        }

        let messages = db
            .get_recent_messages(&project_id, 3)
            .await
            .expect("get");
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_token_count() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "test4", "Desc", None)
            .await
            .expect("create");

        for _ in 0..3 {
            let id = uuid::Uuid::new_v4().to_string();
            db.insert_message(&id, &project_id, "user", "text", None, 100)
                .await
                .expect("insert");
        }

        let total = db
            .get_total_token_count(&project_id)
            .await
            .expect("count");
        assert_eq!(total, 300);
    }

    #[tokio::test]
    async fn test_get_recent_messages_chrono_order() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "order-test", "Desc", None)
            .await
            .expect("create");

        for i in 0..5i64 {
            let id = uuid::Uuid::new_v4().to_string();
            db.insert_message(&id, &project_id, "user", &format!("msg{i}"), None, 10)
                .await
                .expect("insert");
        }

        let messages = db
            .get_recent_messages_chrono(&project_id, 3)
            .await
            .expect("get");
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_get_message_count() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "count-test", "Desc", None)
            .await
            .expect("create");

        assert_eq!(db.get_message_count(&project_id).await.expect("count"), 0);

        for i in 0..3i64 {
            let id = uuid::Uuid::new_v4().to_string();
            db.insert_message(&id, &project_id, "user", &format!("msg{i}"), None, 10)
                .await
                .expect("insert");
        }

        assert_eq!(db.get_message_count(&project_id).await.expect("count"), 3);
    }

    #[tokio::test]
    async fn test_archive_and_summarize() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "archive-test", "Desc", None)
            .await
            .expect("create");

        for i in 0..5i64 {
            let id = uuid::Uuid::new_v4().to_string();
            db.insert_message(&id, &project_id, "user", &format!("msg{i}"), None, 10)
                .await
                .expect("insert");
        }

        assert_eq!(db.get_message_count(&project_id).await.expect("count"), 5);

        let future_ts = "9999-12-31 23:59:59";
        let summary_id = uuid::Uuid::new_v4().to_string();
        let archived = db
            .archive_and_summarize(&project_id, future_ts, &summary_id, "Summary of conversation")
            .await
            .expect("archive");

        assert_eq!(archived, 5);
        assert_eq!(db.get_message_count(&project_id).await.expect("count"), 1);
    }

    #[tokio::test]
    async fn test_discord_channel_lookup() {
        let (db, _dir) = test_db().await;
        let project_id = uuid::Uuid::new_v4().to_string();
        db.create_project(&project_id, "discord-test", "Desc", None)
            .await
            .expect("create");

        let result = db
            .get_project_by_discord_channel("channel-123")
            .await
            .expect("lookup");
        assert!(result.is_none());

        db.set_project_discord_channel(&project_id, "channel-123", "guild-456")
            .await
            .expect("set");

        let result = db
            .get_project_by_discord_channel("channel-123")
            .await
            .expect("lookup");
        assert!(result.is_some());
        assert_eq!(result.expect("project").id, project_id);
    }

    #[tokio::test]
    async fn test_template_crud() {
        let (db, _dir) = test_db().await;
        let id = uuid::Uuid::new_v4().to_string();

        db.create_template(&id, "nuxt-cloudflare", "Nuxt 3 + Cloudflare", "# Template content", "typescript")
            .await
            .expect("create");

        let tmpl = db
            .get_template("nuxt-cloudflare")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(tmpl.name, "nuxt-cloudflare");
        assert_eq!(tmpl.language, "typescript");

        let all = db.list_templates(None).await.expect("list");
        assert!(!all.is_empty());

        let ts_templates = db
            .list_templates(Some("typescript"))
            .await
            .expect("list by lang");
        assert!(!ts_templates.is_empty());

        let deleted = db
            .delete_template("nuxt-cloudflare")
            .await
            .expect("delete");
        assert!(deleted);

        let result = db
            .get_template("nuxt-cloudflare")
            .await
            .expect("get after delete");
        assert!(result.is_none());
    }
}
