//! Password reset token database operations.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::models::PasswordResetToken;

const RESET_TOKEN_DURATION_HOURS: i64 = 1;

/// Create the password_reset_tokens table and indexes.
pub async fn create_password_reset_tokens_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_reset_tokens_user ON password_reset_tokens(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_reset_tokens_hash ON password_reset_tokens(token_hash)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Create a new password reset token.
pub async fn create_reset_token(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    token_hash: &str,
) -> Result<PasswordResetToken, sqlx::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(RESET_TOKEN_DURATION_HOURS);

    sqlx::query_as::<_, PasswordResetToken>(
        r#"INSERT INTO password_reset_tokens (id, user_id, token_hash, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(now.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .fetch_one(pool)
    .await
}

/// Get a reset token by its hash.
pub async fn get_reset_token_by_hash(
    pool: &SqlitePool,
    token_hash: &str,
) -> Result<Option<PasswordResetToken>, sqlx::Error> {
    sqlx::query_as::<_, PasswordResetToken>(
        "SELECT * FROM password_reset_tokens WHERE token_hash = ? AND used_at IS NULL",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Mark a reset token as used.
pub async fn mark_reset_token_used(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = ? WHERE id = ? AND used_at IS NULL",
    )
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Invalidate all pending reset tokens for a user.
pub async fn invalidate_user_reset_tokens(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = ? WHERE user_id = ? AND used_at IS NULL",
    )
    .bind(&now)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Delete expired reset tokens.
pub async fn cleanup_expired_reset_tokens(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < ?")
        .bind(&now)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

/// Check if a token hash exists (used or not).
pub async fn token_hash_exists(pool: &SqlitePool, token_hash: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM password_reset_tokens WHERE token_hash = ?",
    )
    .bind(token_hash)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
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
        create_password_reset_tokens_table(&pool).await.expect("create reset tokens table");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-1', 'test@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create test user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_and_get_reset_token() {
        let (pool, _dir) = test_pool().await;

        let token = create_reset_token(&pool, "token-1", "user-1", "hash-abc123")
            .await
            .expect("create");

        assert_eq!(token.id, "token-1");
        assert_eq!(token.user_id, "user-1");
        assert!(!token.is_used());

        let fetched = get_reset_token_by_hash(&pool, "hash-abc123")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.id, "token-1");
    }

    #[tokio::test]
    async fn test_mark_token_used() {
        let (pool, _dir) = test_pool().await;

        create_reset_token(&pool, "token-2", "user-1", "hash-xyz")
            .await
            .expect("create");

        let marked = mark_reset_token_used(&pool, "token-2").await.expect("mark");
        assert!(marked);

        let fetched = get_reset_token_by_hash(&pool, "hash-xyz")
            .await
            .expect("get");
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_user_tokens() {
        let (pool, _dir) = test_pool().await;

        for i in 0..3 {
            create_reset_token(&pool, &format!("token-{i}"), "user-1", &format!("hash-{i}"))
                .await
                .expect("create");
        }

        let count = invalidate_user_reset_tokens(&pool, "user-1")
            .await
            .expect("invalidate");
        assert_eq!(count, 3);

        for i in 0..3 {
            let fetched = get_reset_token_by_hash(&pool, &format!("hash-{i}"))
                .await
                .expect("get");
            assert!(fetched.is_none());
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let (pool, _dir) = test_pool().await;

        sqlx::query(
            "INSERT INTO password_reset_tokens (id, user_id, token_hash, created_at, expires_at) VALUES ('expired-1', 'user-1', 'hash-expired', datetime('now'), datetime('now', '-1 hour'))",
        )
        .execute(&pool)
        .await
        .expect("create expired token");

        create_reset_token(&pool, "valid-1", "user-1", "hash-valid")
            .await
            .expect("create valid");

        let cleaned = cleanup_expired_reset_tokens(&pool).await.expect("cleanup");
        assert_eq!(cleaned, 1);

        let exists = token_hash_exists(&pool, "hash-expired").await.expect("check");
        assert!(!exists);

        let exists = token_hash_exists(&pool, "hash-valid").await.expect("check");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_token_hash_exists() {
        let (pool, _dir) = test_pool().await;

        let exists = token_hash_exists(&pool, "nonexistent").await.expect("check");
        assert!(!exists);

        create_reset_token(&pool, "token-exists", "user-1", "hash-exists")
            .await
            .expect("create");

        let exists = token_hash_exists(&pool, "hash-exists").await.expect("check");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_token_expiry_time() {
        let (pool, _dir) = test_pool().await;

        let token = create_reset_token(&pool, "token-expiry", "user-1", "hash-expiry")
            .await
            .expect("create");

        let created = chrono::DateTime::parse_from_rfc3339(&token.created_at).expect("parse created");
        let expires = chrono::DateTime::parse_from_rfc3339(&token.expires_at).expect("parse expires");

        let duration = expires.signed_duration_since(created);
        assert_eq!(duration.num_hours(), RESET_TOKEN_DURATION_HOURS);
    }
}
