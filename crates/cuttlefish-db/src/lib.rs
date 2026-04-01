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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    async fn test_db() -> Database {
        let tmp = NamedTempFile::new().expect("temp file");
        Database::open(tmp.path()).await.expect("db open")
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let db = test_db().await;
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
        let db = test_db().await;
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
        let db = test_db().await;
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
        let db = test_db().await;
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
}
