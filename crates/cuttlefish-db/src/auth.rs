//! Authentication-related database operations.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::User;

/// Create the users table and indexes.
pub async fn create_users_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    email_verified_at TEXT,
    last_login_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Create a new user in the database.
pub async fn create_user(
    pool: &SqlitePool,
    id: &str,
    email: &str,
    password_hash: &str,
    display_name: Option<&str>,
) -> Result<User, sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id, email, password_hash, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *"#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await
}

/// Get a user by ID.
pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Get a user by email.
pub async fn get_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
}

/// Update a user's display name.
pub async fn update_user_display_name(
    pool: &SqlitePool,
    id: &str,
    display_name: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET display_name = ?, updated_at = ? WHERE id = ?")
        .bind(display_name)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update a user's password hash.
pub async fn update_user_password(
    pool: &SqlitePool,
    id: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(password_hash)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Mark a user's email as verified.
pub async fn verify_user_email(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET email_verified_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Update a user's last login timestamp.
pub async fn update_last_login(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Deactivate a user account.
pub async fn deactivate_user(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET is_active = 0, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Reactivate a user account.
pub async fn reactivate_user(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query("UPDATE users SET is_active = 1, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete a user by ID.
pub async fn delete_user(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// List all active users.
pub async fn list_active_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE is_active = 1 ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

/// Check if an email is already registered.
pub async fn email_exists(pool: &SqlitePool, email: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;

    Ok(result > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");
        create_users_table(&pool).await.expect("create table");
        (pool, dir)
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let (pool, _dir) = test_pool().await;

        let user = create_user(
            &pool,
            "user-123",
            "test@example.com",
            "$argon2id$hash",
            Some("Test User"),
        )
        .await
        .expect("create user");

        assert_eq!(user.id, "user-123");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert!(user.is_active());

        let fetched = get_user_by_id(&pool, "user-123")
            .await
            .expect("get user")
            .expect("user exists");
        assert_eq!(fetched.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-456", "email@test.com", "hash", None)
            .await
            .expect("create");

        let user = get_user_by_email(&pool, "email@test.com")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(user.id, "user-456");

        let not_found = get_user_by_email(&pool, "nonexistent@test.com")
            .await
            .expect("get");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_email_uniqueness() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-1", "unique@test.com", "hash", None)
            .await
            .expect("create first");

        let result = create_user(&pool, "user-2", "unique@test.com", "hash", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_display_name() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-789", "update@test.com", "hash", None)
            .await
            .expect("create");

        let updated = update_user_display_name(&pool, "user-789", Some("New Name"))
            .await
            .expect("update");
        assert!(updated);

        let user = get_user_by_id(&pool, "user-789")
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(user.display_name, Some("New Name".to_string()));
    }

    #[tokio::test]
    async fn test_deactivate_and_reactivate() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-active", "active@test.com", "hash", None)
            .await
            .expect("create");

        deactivate_user(&pool, "user-active")
            .await
            .expect("deactivate");

        let user = get_user_by_id(&pool, "user-active")
            .await
            .expect("get")
            .expect("exists");
        assert!(!user.is_active());

        reactivate_user(&pool, "user-active")
            .await
            .expect("reactivate");

        let user = get_user_by_id(&pool, "user-active")
            .await
            .expect("get")
            .expect("exists");
        assert!(user.is_active());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-delete", "delete@test.com", "hash", None)
            .await
            .expect("create");

        let deleted = delete_user(&pool, "user-delete").await.expect("delete");
        assert!(deleted);

        let user = get_user_by_id(&pool, "user-delete").await.expect("get");
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_email_exists() {
        let (pool, _dir) = test_pool().await;

        assert!(!email_exists(&pool, "check@test.com").await.expect("check"));

        create_user(&pool, "user-check", "check@test.com", "hash", None)
            .await
            .expect("create");

        assert!(email_exists(&pool, "check@test.com").await.expect("check"));
    }

    #[tokio::test]
    async fn test_verify_email() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-verify", "verify@test.com", "hash", None)
            .await
            .expect("create");

        let user = get_user_by_id(&pool, "user-verify")
            .await
            .expect("get")
            .expect("exists");
        assert!(user.email_verified_at.is_none());

        verify_user_email(&pool, "user-verify")
            .await
            .expect("verify");

        let user = get_user_by_id(&pool, "user-verify")
            .await
            .expect("get")
            .expect("exists");
        assert!(user.email_verified_at.is_some());
    }

    #[tokio::test]
    async fn test_update_last_login() {
        let (pool, _dir) = test_pool().await;

        create_user(&pool, "user-login", "login@test.com", "hash", None)
            .await
            .expect("create");

        let user = get_user_by_id(&pool, "user-login")
            .await
            .expect("get")
            .expect("exists");
        assert!(user.last_login_at.is_none());

        update_last_login(&pool, "user-login")
            .await
            .expect("update");

        let user = get_user_by_id(&pool, "user-login")
            .await
            .expect("get")
            .expect("exists");
        assert!(user.last_login_at.is_some());
    }
}
