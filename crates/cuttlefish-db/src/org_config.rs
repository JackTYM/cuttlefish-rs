//! Organization-level configuration management.
//!
//! Provides org-wide settings that can be inherited by projects,
//! with support for enforced vs. default configurations.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::organization::{OrgError, OrgRole, get_organization};

/// Model configuration for an organization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default model ID.
    pub model_id: Option<String>,
    /// Default provider.
    pub provider: Option<String>,
    /// Temperature setting.
    pub temperature: Option<f64>,
    /// Max tokens.
    pub max_tokens: Option<i64>,
}

/// Sandbox limits configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxLimits {
    /// Memory limit in MB.
    pub memory_mb: Option<i64>,
    /// CPU limit (cores).
    pub cpu_cores: Option<f64>,
    /// Disk limit in GB.
    pub disk_gb: Option<i64>,
    /// Network access allowed.
    pub network_enabled: Option<bool>,
}

/// Gate configuration for approval workflows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateConfig {
    /// Require approval for deployments.
    pub require_deploy_approval: bool,
    /// Require approval for destructive operations.
    pub require_destructive_approval: bool,
    /// Auto-approve for owners.
    pub auto_approve_owners: bool,
}

/// Full organization configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrgConfig {
    /// Default model configuration.
    pub model_config: Option<ModelConfig>,
    /// Allowed model providers (None = all allowed).
    pub allowed_providers: Option<Vec<String>>,
    /// Allowed model IDs (None = all allowed).
    pub allowed_models: Option<Vec<String>>,
    /// Default sandbox limits.
    pub sandbox_limits: Option<SandboxLimits>,
    /// Gate configuration.
    pub gate_config: Option<GateConfig>,
    /// Maximum projects allowed in org.
    pub max_projects: Option<i64>,
    /// Maximum members allowed in org.
    pub max_members: Option<i64>,
    /// Shared template IDs available to all org projects.
    pub shared_templates: Vec<String>,
    /// Whether projects can override org settings.
    pub allow_project_overrides: bool,
    /// Custom metadata (JSON).
    pub metadata: Option<String>,
}

impl OrgConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self {
            allow_project_overrides: true,
            ..Default::default()
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }

    /// Check if a provider is allowed.
    pub fn is_provider_allowed(&self, provider: &str) -> bool {
        match &self.allowed_providers {
            Some(providers) => providers.iter().any(|p| p == provider),
            None => true,
        }
    }

    /// Check if a model is allowed.
    pub fn is_model_allowed(&self, model: &str) -> bool {
        match &self.allowed_models {
            Some(models) => models.iter().any(|m| m == model),
            None => true,
        }
    }
}

/// Create the org_configs table.
pub async fn create_org_configs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS org_configs (
    id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL UNIQUE,
    config TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE SET NULL
)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_org_configs_org ON org_configs(org_id)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Get organization configuration.
pub async fn get_org_config(pool: &SqlitePool, org_id: &str) -> Result<OrgConfig, OrgError> {
    let org = get_organization(pool, org_id)
        .await?
        .ok_or(OrgError::NotFound)?;

    let row: Option<(String,)> = sqlx::query_as("SELECT config FROM org_configs WHERE org_id = ?")
        .bind(org_id)
        .fetch_optional(pool)
        .await
        .map_err(OrgError::from)?;

    match row {
        Some((config_json,)) => Ok(OrgConfig::from_json(&config_json)),
        None => Ok(OrgConfig {
            allowed_providers: org.settings().allowed_providers,
            max_projects: org.settings().max_projects,
            shared_templates: org.settings().shared_templates,
            allow_project_overrides: true,
            ..Default::default()
        }),
    }
}

/// Update organization configuration.
///
/// Requires admin or owner role.
pub async fn update_org_config(
    pool: &SqlitePool,
    org_id: &str,
    config: &OrgConfig,
    actor_id: &str,
) -> Result<bool, OrgError> {
    use crate::organization::can_user_access_org;

    let can_update = can_user_access_org(pool, actor_id, org_id, OrgRole::Admin)
        .await
        .map_err(OrgError::from)?;

    if !can_update {
        return Err(OrgError::InsufficientPermissions);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let config_json = config.to_json();

    let result = sqlx::query(
        r#"INSERT INTO org_configs (id, org_id, config, updated_at, updated_by)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(org_id) DO UPDATE SET
            config = excluded.config,
            updated_at = excluded.updated_at,
            updated_by = excluded.updated_by"#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(org_id)
    .bind(&config_json)
    .bind(&now)
    .bind(actor_id)
    .execute(pool)
    .await
    .map_err(OrgError::from)?;

    Ok(result.rows_affected() > 0)
}

/// Merge project config with org config, respecting override rules.
///
/// Returns the effective configuration for a project.
pub fn merge_configs(org_config: &OrgConfig, project_config: &OrgConfig) -> OrgConfig {
    if !org_config.allow_project_overrides {
        return org_config.clone();
    }

    OrgConfig {
        model_config: project_config
            .model_config
            .clone()
            .or(org_config.model_config.clone()),
        allowed_providers: org_config.allowed_providers.clone(),
        allowed_models: org_config.allowed_models.clone(),
        sandbox_limits: project_config
            .sandbox_limits
            .clone()
            .or(org_config.sandbox_limits.clone()),
        gate_config: project_config
            .gate_config
            .clone()
            .or(org_config.gate_config.clone()),
        max_projects: org_config.max_projects,
        max_members: org_config.max_members,
        shared_templates: {
            let mut templates = org_config.shared_templates.clone();
            templates.extend(project_config.shared_templates.clone());
            templates.sort();
            templates.dedup();
            templates
        },
        allow_project_overrides: org_config.allow_project_overrides,
        metadata: project_config
            .metadata
            .clone()
            .or(org_config.metadata.clone()),
    }
}

/// Validate that a configuration respects org constraints.
pub fn validate_config_against_org(
    org_config: &OrgConfig,
    project_config: &OrgConfig,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(ref model_config) = project_config.model_config {
        if let Some(ref provider) = model_config.provider
            && !org_config.is_provider_allowed(provider)
        {
            errors.push(format!(
                "Provider '{}' is not allowed by organization",
                provider
            ));
        }
        if let Some(ref model_id) = model_config.model_id
            && !org_config.is_model_allowed(model_id)
        {
            errors.push(format!(
                "Model '{}' is not allowed by organization",
                model_id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::create_users_table;
    use crate::organization::{create_organization, create_organizations_tables};
    use tempfile::TempDir;

    async fn test_pool() -> (SqlitePool, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await.expect("connect");

        create_users_table(&pool).await.expect("create users table");
        create_organizations_tables(&pool)
            .await
            .expect("create org tables");
        create_org_configs_table(&pool)
            .await
            .expect("create config table");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-alice', 'alice@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create user");

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ('user-bob', 'bob@example.com', 'hash', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("create user");

        (pool, dir)
    }

    #[tokio::test]
    async fn test_get_org_config_default() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create org");

        let config = get_org_config(&pool, "org-1").await.expect("get config");
        assert!(config.model_config.is_none());
        assert!(config.allow_project_overrides);
    }

    #[tokio::test]
    async fn test_update_org_config() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create org");

        let mut config = OrgConfig::new();
        config.model_config = Some(ModelConfig {
            model_id: Some("claude-sonnet".to_string()),
            provider: Some("anthropic".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4096),
        });
        config.allowed_providers = Some(vec!["anthropic".to_string(), "openai".to_string()]);
        config.max_projects = Some(10);

        update_org_config(&pool, "org-1", &config, "user-alice")
            .await
            .expect("update config");

        let loaded = get_org_config(&pool, "org-1").await.expect("get config");
        assert!(loaded.model_config.is_some());
        assert_eq!(
            loaded
                .model_config
                .as_ref()
                .and_then(|m| m.model_id.as_ref()),
            Some(&"claude-sonnet".to_string())
        );
        assert_eq!(loaded.max_projects, Some(10));
    }

    #[tokio::test]
    async fn test_update_config_insufficient_permissions() {
        let (pool, _dir) = test_pool().await;

        create_organization(&pool, "org-1", "Test Org", "user-alice")
            .await
            .expect("create org");

        let config = OrgConfig::new();
        let result = update_org_config(&pool, "org-1", &config, "user-bob").await;
        assert!(matches!(result, Err(OrgError::InsufficientPermissions)));
    }

    #[tokio::test]
    async fn test_is_provider_allowed() {
        let mut config = OrgConfig::new();
        config.allowed_providers = Some(vec!["anthropic".to_string(), "openai".to_string()]);

        assert!(config.is_provider_allowed("anthropic"));
        assert!(config.is_provider_allowed("openai"));
        assert!(!config.is_provider_allowed("google"));

        config.allowed_providers = None;
        assert!(config.is_provider_allowed("any-provider"));
    }

    #[tokio::test]
    async fn test_is_model_allowed() {
        let mut config = OrgConfig::new();
        config.allowed_models = Some(vec!["claude-sonnet".to_string(), "gpt-4".to_string()]);

        assert!(config.is_model_allowed("claude-sonnet"));
        assert!(config.is_model_allowed("gpt-4"));
        assert!(!config.is_model_allowed("claude-opus"));

        config.allowed_models = None;
        assert!(config.is_model_allowed("any-model"));
    }

    #[test]
    fn test_merge_configs() {
        let mut org_config = OrgConfig::new();
        org_config.model_config = Some(ModelConfig {
            model_id: Some("org-default".to_string()),
            provider: Some("anthropic".to_string()),
            temperature: None,
            max_tokens: None,
        });
        org_config.allowed_providers = Some(vec!["anthropic".to_string()]);
        org_config.shared_templates = vec!["org-tmpl".to_string()];

        let mut project_config = OrgConfig::new();
        project_config.model_config = Some(ModelConfig {
            model_id: Some("project-model".to_string()),
            provider: None,
            temperature: Some(0.5),
            max_tokens: None,
        });
        project_config.shared_templates = vec!["proj-tmpl".to_string()];

        let merged = merge_configs(&org_config, &project_config);

        assert_eq!(
            merged
                .model_config
                .as_ref()
                .and_then(|m| m.model_id.as_ref()),
            Some(&"project-model".to_string())
        );
        assert_eq!(
            merged.allowed_providers,
            Some(vec!["anthropic".to_string()])
        );
        assert!(merged.shared_templates.contains(&"org-tmpl".to_string()));
        assert!(merged.shared_templates.contains(&"proj-tmpl".to_string()));
    }

    #[test]
    fn test_merge_configs_no_override() {
        let mut org_config = OrgConfig::new();
        org_config.allow_project_overrides = false;
        org_config.model_config = Some(ModelConfig {
            model_id: Some("org-model".to_string()),
            provider: None,
            temperature: None,
            max_tokens: None,
        });

        let mut project_config = OrgConfig::new();
        project_config.model_config = Some(ModelConfig {
            model_id: Some("project-model".to_string()),
            provider: None,
            temperature: None,
            max_tokens: None,
        });

        let merged = merge_configs(&org_config, &project_config);

        assert_eq!(
            merged
                .model_config
                .as_ref()
                .and_then(|m| m.model_id.as_ref()),
            Some(&"org-model".to_string())
        );
    }

    #[test]
    fn test_validate_config_against_org() {
        let mut org_config = OrgConfig::new();
        org_config.allowed_providers = Some(vec!["anthropic".to_string()]);
        org_config.allowed_models = Some(vec!["claude-sonnet".to_string()]);

        let mut valid_project = OrgConfig::new();
        valid_project.model_config = Some(ModelConfig {
            model_id: Some("claude-sonnet".to_string()),
            provider: Some("anthropic".to_string()),
            temperature: None,
            max_tokens: None,
        });

        assert!(validate_config_against_org(&org_config, &valid_project).is_ok());

        let mut invalid_project = OrgConfig::new();
        invalid_project.model_config = Some(ModelConfig {
            model_id: Some("gpt-4".to_string()),
            provider: Some("openai".to_string()),
            temperature: None,
            max_tokens: None,
        });

        let result = validate_config_against_org(&org_config, &invalid_project);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_org_config_serialization() {
        let mut config = OrgConfig::new();
        config.model_config = Some(ModelConfig {
            model_id: Some("test-model".to_string()),
            provider: Some("test-provider".to_string()),
            temperature: Some(0.8),
            max_tokens: Some(2048),
        });
        config.sandbox_limits = Some(SandboxLimits {
            memory_mb: Some(4096),
            cpu_cores: Some(2.0),
            disk_gb: Some(20),
            network_enabled: Some(true),
        });
        config.gate_config = Some(GateConfig {
            require_deploy_approval: true,
            require_destructive_approval: true,
            auto_approve_owners: true,
        });

        let json = config.to_json();
        let restored = OrgConfig::from_json(&json);

        assert_eq!(
            restored
                .model_config
                .as_ref()
                .and_then(|m| m.model_id.as_ref()),
            Some(&"test-model".to_string())
        );
        assert_eq!(
            restored.sandbox_limits.as_ref().and_then(|s| s.memory_mb),
            Some(4096)
        );
        assert_eq!(
            restored
                .gate_config
                .as_ref()
                .map(|g| g.require_deploy_approval),
            Some(true)
        );
    }
}
