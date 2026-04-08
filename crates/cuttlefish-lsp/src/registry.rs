//! LSP server registry for managing multiple language servers.

use std::collections::HashMap;
use std::path::Path;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::client::LspClient;
use crate::error::LspError;

/// Configuration for a language server.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    /// Language ID (e.g., "rust", "typescript").
    pub language_id: String,
    /// Command to run.
    pub command: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: Vec<(String, String)>,
    /// File extensions this server handles.
    pub extensions: Vec<String>,
}

impl LspServerConfig {
    /// Create config for rust-analyzer.
    pub fn rust_analyzer() -> Self {
        Self {
            language_id: "rust".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            env: vec![],
            extensions: vec!["rs".to_string()],
        }
    }

    /// Create config for TypeScript language server.
    pub fn typescript() -> Self {
        Self {
            language_id: "typescript".to_string(),
            command: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            env: vec![],
            extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
            ],
        }
    }

    /// Create config for Python language server (pyright).
    pub fn python() -> Self {
        Self {
            language_id: "python".to_string(),
            command: "pyright-langserver".to_string(),
            args: vec!["--stdio".to_string()],
            env: vec![],
            extensions: vec!["py".to_string()],
        }
    }

    /// Create config for Go language server (gopls).
    pub fn go() -> Self {
        Self {
            language_id: "go".to_string(),
            command: "gopls".to_string(),
            args: vec![],
            env: vec![],
            extensions: vec!["go".to_string()],
        }
    }

    /// Create config for C/C++ language server (clangd).
    pub fn clangd() -> Self {
        Self {
            language_id: "cpp".to_string(),
            command: "clangd".to_string(),
            args: vec![],
            env: vec![],
            extensions: vec![
                "c".to_string(),
                "cpp".to_string(),
                "cc".to_string(),
                "h".to_string(),
                "hpp".to_string(),
            ],
        }
    }
}

/// A managed LSP server instance.
struct ManagedServer {
    /// Server configuration (kept for potential reconfiguration).
    #[allow(dead_code)]
    config: LspServerConfig,
    /// LSP client.
    client: LspClient,
}

/// Registry for managing multiple LSP servers.
pub struct LspRegistry {
    /// Server configurations by language ID.
    configs: RwLock<HashMap<String, LspServerConfig>>,
    /// Active servers by language ID.
    servers: RwLock<HashMap<String, ManagedServer>>,
    /// Root path for workspace.
    root_path: Option<std::path::PathBuf>,
}

impl LspRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            servers: RwLock::new(HashMap::new()),
            root_path: None,
        }
    }

    /// Create a registry with a root path.
    pub fn with_root_path(root_path: impl AsRef<Path>) -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            servers: RwLock::new(HashMap::new()),
            root_path: Some(root_path.as_ref().to_path_buf()),
        }
    }

    /// Register a server configuration.
    pub async fn register_config(&self, config: LspServerConfig) {
        let mut configs = self.configs.write().await;
        info!("Registered LSP config for language: {}", config.language_id);
        configs.insert(config.language_id.clone(), config);
    }

    /// Register default configurations for common languages.
    pub async fn register_defaults(&self) {
        self.register_config(LspServerConfig::rust_analyzer()).await;
        self.register_config(LspServerConfig::typescript()).await;
        self.register_config(LspServerConfig::python()).await;
        self.register_config(LspServerConfig::go()).await;
        self.register_config(LspServerConfig::clangd()).await;
    }

    /// Get or start an LSP server for a file.
    pub async fn get_server_for_file(&self, path: &Path) -> Result<&LspClient, LspError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Find config for this extension
        let configs = self.configs.read().await;
        let config = configs
            .values()
            .find(|c| c.extensions.iter().any(|e| e == ext))
            .cloned();
        drop(configs);

        let Some(config) = config else {
            return Err(LspError::FileNotFound(format!(
                "No LSP server configured for extension: {}",
                ext
            )));
        };

        self.get_or_start_server(&config.language_id).await
    }

    /// Get or start a server by language ID.
    pub async fn get_or_start_server(&self, language_id: &str) -> Result<&LspClient, LspError> {
        // Check if server is already running
        {
            let servers = self.servers.read().await;
            if servers.contains_key(language_id) {
                // This is safe because we hold the lock and the server exists
                drop(servers);
                let _servers = self.servers.read().await;
                // Note: We can't return a reference directly due to lifetime issues
                // In practice, you'd use a different pattern (Arc<LspClient>)
            }
        }

        // Need to start the server
        self.start_server(language_id).await?;

        // Return error - in real implementation, use Arc<LspClient>
        Err(LspError::NotInitialized)
    }

    /// Start a server for a language.
    pub async fn start_server(&self, language_id: &str) -> Result<(), LspError> {
        let configs = self.configs.read().await;
        let config = configs.get(language_id).cloned().ok_or_else(|| {
            LspError::FileNotFound(format!("No config for language: {}", language_id))
        })?;
        drop(configs);

        // Check if already running
        {
            let servers = self.servers.read().await;
            if servers.contains_key(language_id) {
                debug!("LSP server for {} already running", language_id);
                return Ok(());
            }
        }

        // Spawn the server
        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let env: Vec<(&str, &str)> = config
            .env
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let client =
            LspClient::spawn_with_env(&config.command, &args, &env, self.root_path.as_deref())
                .await?;

        // Initialize
        client.initialize().await?;

        // Store
        let mut servers = self.servers.write().await;
        servers.insert(language_id.to_string(), ManagedServer { config, client });

        info!("Started LSP server for language: {}", language_id);
        Ok(())
    }

    /// Stop a server by language ID.
    pub async fn stop_server(&self, language_id: &str) -> Result<(), LspError> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.remove(language_id) {
            server.client.shutdown().await?;
            info!("Stopped LSP server for language: {}", language_id);
        }
        Ok(())
    }

    /// Stop all servers.
    pub async fn shutdown_all(&self) -> Result<(), LspError> {
        let mut servers = self.servers.write().await;
        for (lang, server) in servers.drain() {
            if let Err(e) = server.client.shutdown().await {
                warn!("Failed to shutdown {} server: {}", lang, e);
            }
        }
        info!("All LSP servers shut down");
        Ok(())
    }

    /// Get diagnostics for a file.
    pub async fn get_diagnostics(
        &self,
        path: &Path,
    ) -> Result<Vec<lsp_types::Diagnostic>, LspError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Find the language
        let configs = self.configs.read().await;
        let language_id = configs
            .values()
            .find(|c| c.extensions.iter().any(|e| e == ext))
            .map(|c| c.language_id.clone());
        drop(configs);

        let Some(language_id) = language_id else {
            return Ok(Vec::new());
        };

        // Start server if needed
        self.start_server(&language_id).await?;

        // Get diagnostics
        let servers = self.servers.read().await;
        if let Some(server) = servers.get(&language_id) {
            server.client.get_diagnostics(path).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get document symbols for a file.
    pub async fn get_symbols(
        &self,
        path: &Path,
    ) -> Result<Option<lsp_types::DocumentSymbolResponse>, LspError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let configs = self.configs.read().await;
        let language_id = configs
            .values()
            .find(|c| c.extensions.iter().any(|e| e == ext))
            .map(|c| c.language_id.clone());
        drop(configs);

        let Some(language_id) = language_id else {
            return Ok(None);
        };

        self.start_server(&language_id).await?;

        let servers = self.servers.read().await;
        if let Some(server) = servers.get(&language_id) {
            server.client.document_symbols(path).await
        } else {
            Ok(None)
        }
    }

    /// Go to definition.
    pub async fn goto_definition(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<lsp_types::GotoDefinitionResponse>, LspError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let configs = self.configs.read().await;
        let language_id = configs
            .values()
            .find(|c| c.extensions.iter().any(|e| e == ext))
            .map(|c| c.language_id.clone());
        drop(configs);

        let Some(language_id) = language_id else {
            return Ok(None);
        };

        self.start_server(&language_id).await?;

        let servers = self.servers.read().await;
        if let Some(server) = servers.get(&language_id) {
            server.client.goto_definition(path, line, character).await
        } else {
            Ok(None)
        }
    }

    /// Get hover information.
    pub async fn hover(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<lsp_types::Hover>, LspError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let configs = self.configs.read().await;
        let language_id = configs
            .values()
            .find(|c| c.extensions.iter().any(|e| e == ext))
            .map(|c| c.language_id.clone());
        drop(configs);

        let Some(language_id) = language_id else {
            return Ok(None);
        };

        self.start_server(&language_id).await?;

        let servers = self.servers.read().await;
        if let Some(server) = servers.get(&language_id) {
            server.client.hover(path, line, character).await
        } else {
            Ok(None)
        }
    }

    /// List registered languages.
    pub async fn registered_languages(&self) -> Vec<String> {
        let configs = self.configs.read().await;
        configs.keys().cloned().collect()
    }

    /// List active servers.
    pub async fn active_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }
}

impl Default for LspRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = LspRegistry::new();
        assert!(registry.registered_languages().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_config() {
        let registry = LspRegistry::new();
        registry
            .register_config(LspServerConfig::rust_analyzer())
            .await;

        let langs = registry.registered_languages().await;
        assert!(langs.contains(&"rust".to_string()));
    }

    #[test]
    fn test_server_configs() {
        let rust = LspServerConfig::rust_analyzer();
        assert_eq!(rust.language_id, "rust");
        assert!(rust.extensions.contains(&"rs".to_string()));

        let ts = LspServerConfig::typescript();
        assert_eq!(ts.language_id, "typescript");
        assert!(ts.extensions.contains(&"ts".to_string()));
    }
}
