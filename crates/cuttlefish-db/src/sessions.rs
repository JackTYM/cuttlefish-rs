//! Session management database operations.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::models::Session;

const MAX_SESSIONS_PER_USER: i64 = 10;
const REFRESH_TOKEN_DURATION_DAYS: i64 = 30;

/// Create the sessions table and indexes.
pub async fn create_sessions_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    user_agent TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(refresh_token_hash)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Create a new session for a user.
pub async fn create_session(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    refresh_token_hash: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<Session, sqlx::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::days(REFRESH_TOKEN_DURATION_DAYS);

    sqlx::query_as::<_, Session>(
        r#"INSERT INTO sessions (id, user_id, refresh_token_hash, user_agent, ip_address, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(refresh_token_hash)
    .bind(user_agent)
    .bind(ip_address)
    .bind(now.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .fetch_one(pool)
    .await
}

/// Get a session by ID.
pub async fn get_session_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get a session by refresh token hash.
pub async fn get_session_by_token_hash(
    pool: &SqlitePool,
    token_hash: &str,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE refresh_token_hash = ? AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Revoke a session by ID.
pub async fn revoke_session(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE sessions SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

/// Revoke all sessions for a user.
pub async fn revoke_all_user_sessions(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result =
        sqlx::query("UPDATE sessions SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL")
            .bind(&now)
            .bind(user_id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected())
}

/// Update the refresh token hash for a session (token rotation).
pub async fn update_session_token(
    pool: &SqlitePool,
    id: &str,
    new_token_hash: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE sessions SET refresh_token_hash = ? WHERE id = ? AND revoked_at IS NULL",
    )
    .bind(new_token_hash)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get the count of active sessions for a user.
pub async fn count_user_sessions(pool: &SqlitePool, user_id: &str) -> Result<i64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions WHERE user_id = ? AND revoked_at IS NULL AND expires_at > ?",
    )
    .bind(user_id)
    .bind(&now)
    .fetch_one(pool)
    .await
}

/// Get all active sessions for a user.
pub async fn get_user_sessions(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<Session>, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE user_id = ? AND revoked_at IS NULL AND expires_at > ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(&now)
    .fetch_all(pool)
    .await
}

/// Delete the oldest session for a user (to enforce max sessions limit).
pub async fn delete_oldest_session(pool: &SqlitePool, user_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"DELETE FROM sessions WHERE id = (
            SELECT id FROM sessions 
            WHERE user_id = ? AND revoked_at IS NULL 
            ORDER BY created_at ASC 
            LIMIT 1
        )"#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Cleanup expired sessions.
pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
        .bind(&now)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

/// Check if user has reached max sessions and delete oldest if needed.
pub async fn enforce_session_limit(pool: &SqlitePool, user_id: &str) -> Result<(), sqlx::Error> {
    let count = count_user_sessions(pool, user_id).await?;
    if count >= MAX_SESSIONS_PER_USER {
        delete_oldest_session(pool, user_id).await?;
    }
    Ok(())
}

/// Check if a token hash was previously used (for reuse detection).
pub async fn was_token_previously_used(
    pool: &SqlitePool,
    token_hash: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sessions WHERE refresh_token_hash = ? AND revoked_at IS NOT NULL",
    )
    .bind(token_hash)
    .fetch_one(pool)
    .await?;

    Ok(result > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_users_table;
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");
        create_users_table(&pool).await.expect("create users table");
        create_sessions_table(&pool)
            .await
            .expect("create sessions table");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-1', 'test@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create test user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let (pool, _dir) = test_pool().await;

        let session = create_session(
            &pool,
            "session-1",
            "user-1",
            "token-hash-123",
            Some("Mozilla/5.0"),
            Some("192.168.1.1"),
        )
        .await
        .expect("create session");

        assert_eq!(session.id, "session-1");
        assert_eq!(session.user_id, "user-1");
        assert!(!session.is_revoked());

        let fetched = get_session_by_id(&pool, "session-1")
            .await
            .expect("get session")
            .expect("session exists");
        assert_eq!(fetched.refresh_token_hash, "token-hash-123");
    }

    #[tokio::test]
    async fn test_get_session_by_token_hash() {
        let (pool, _dir) = test_pool().await;

        create_session(&pool, "session-2", "user-1", "unique-hash", None, None)
            .await
            .expect("create");

        let session = get_session_by_token_hash(&pool, "unique-hash")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(session.id, "session-2");

        let not_found = get_session_by_token_hash(&pool, "nonexistent")
            .await
            .expect("get");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_revoke_session() {
        let (pool, _dir) = test_pool().await;

        create_session(&pool, "session-3", "user-1", "hash-3", None, None)
            .await
            .expect("create");

        let revoked = revoke_session(&pool, "session-3").await.expect("revoke");
        assert!(revoked);

        let session = get_session_by_id(&pool, "session-3")
            .await
            .expect("get")
            .expect("exists");
        assert!(session.is_revoked());

        let by_hash = get_session_by_token_hash(&pool, "hash-3")
            .await
            .expect("get");
        assert!(by_hash.is_none());
    }

    #[tokio::test]
    async fn test_revoke_all_user_sessions() {
        let (pool, _dir) = test_pool().await;

        for i in 0..3 {
            create_session(
                &pool,
                &format!("session-{i}"),
                "user-1",
                &format!("hash-{i}"),
                None,
                None,
            )
            .await
            .expect("create");
        }

        let count = revoke_all_user_sessions(&pool, "user-1")
            .await
            .expect("revoke all");
        assert_eq!(count, 3);

        let sessions = get_user_sessions(&pool, "user-1").await.expect("get");
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_update_session_token() {
        let (pool, _dir) = test_pool().await;

        create_session(&pool, "session-rotate", "user-1", "old-hash", None, None)
            .await
            .expect("create");

        let updated = update_session_token(&pool, "session-rotate", "new-hash")
            .await
            .expect("update");
        assert!(updated);

        let session = get_session_by_id(&pool, "session-rotate")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(session.refresh_token_hash, "new-hash");
    }

    #[tokio::test]
    async fn test_session_count_and_limit() {
        let (pool, _dir) = test_pool().await;

        for i in 0..5 {
            create_session(
                &pool,
                &format!("limit-{i}"),
                "user-1",
                &format!("hash-{i}"),
                None,
                None,
            )
            .await
            .expect("create");
        }

        let count = count_user_sessions(&pool, "user-1").await.expect("count");
        assert_eq!(count, 5);

        delete_oldest_session(&pool, "user-1")
            .await
            .expect("delete oldest");

        let count = count_user_sessions(&pool, "user-1").await.expect("count");
        assert_eq!(count, 4);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let (pool, _dir) = test_pool().await;

        sqlx::query(
            "INSERT INTO sessions (id, user_id, refresh_token_hash, created_at, expires_at) VALUES ('expired-1', 'user-1', 'hash', datetime('now'), datetime('now', '-1 day'))",
        )
        .execute(&pool)
        .await
        .expect("create expired session");

        create_session(&pool, "valid-1", "user-1", "valid-hash", None, None)
            .await
            .expect("create valid");

        let cleaned = cleanup_expired_sessions(&pool).await.expect("cleanup");
        assert_eq!(cleaned, 1);

        let valid = get_session_by_id(&pool, "valid-1").await.expect("get");
        assert!(valid.is_some());

        let expired = get_session_by_id(&pool, "expired-1").await.expect("get");
        assert!(expired.is_none());
    }

    #[tokio::test]
    async fn test_was_token_previously_used() {
        let (pool, _dir) = test_pool().await;

        create_session(&pool, "reuse-test", "user-1", "reuse-hash", None, None)
            .await
            .expect("create");

        let was_used = was_token_previously_used(&pool, "reuse-hash")
            .await
            .expect("check");
        assert!(!was_used);

        revoke_session(&pool, "reuse-test").await.expect("revoke");

        let was_used = was_token_previously_used(&pool, "reuse-hash")
            .await
            .expect("check");
        assert!(was_used);
    }
}
