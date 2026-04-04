//! WebUI static file serving for the Cuttlefish dashboard.
//!
//! Serves the Nuxt-generated static files with SPA fallback support.
//! All non-API routes that don't match a static file return `index.html`.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{debug, warn};

/// Configuration for WebUI static file serving.
#[derive(Debug, Clone)]
pub struct WebUiConfig {
    /// Path to the directory containing static files (e.g., `.output/public`).
    pub static_dir: PathBuf,
    /// Whether WebUI serving is enabled.
    pub enabled: bool,
}

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            static_dir: PathBuf::from("cuttlefish-web/.output/public"),
            enabled: true,
        }
    }
}

impl WebUiConfig {
    /// Create a new WebUI config with the given static directory.
    pub fn new(static_dir: impl Into<PathBuf>) -> Self {
        Self {
            static_dir: static_dir.into(),
            enabled: true,
        }
    }

    /// Create a disabled WebUI config.
    pub fn disabled() -> Self {
        Self {
            static_dir: PathBuf::new(),
            enabled: false,
        }
    }

    /// Check if the static directory exists and contains index.html.
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let index_path = self.static_dir.join("index.html");
        index_path.exists()
    }
}

/// State for the WebUI service.
#[derive(Clone)]
pub struct WebUiState {
    /// Configuration for static file serving.
    pub config: WebUiConfig,
}

impl WebUiState {
    /// Create a new WebUI state with the given config.
    pub fn new(config: WebUiConfig) -> Self {
        Self { config }
    }
}

/// Handler for serving static files with SPA fallback.
///
/// Tries to serve the requested file from the static directory.
/// If the file doesn't exist and the request doesn't look like a file (no extension),
/// serves `index.html` for SPA client-side routing.
pub async fn webui_handler(State(state): State<WebUiState>, uri: Uri) -> Response {
    let path = uri.path();

    // Skip API routes - they should be handled by other routers
    if path.starts_with("/api/") || path.starts_with("/ws") || path == "/health" {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    if !state.config.enabled {
        return (StatusCode::SERVICE_UNAVAILABLE, "WebUI not enabled").into_response();
    }

    let static_dir = &state.config.static_dir;

    // Try to serve the exact file first
    let file_path = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    let full_path = static_dir.join(&file_path);

    debug!(path = %path, full_path = %full_path.display(), "Serving static file");

    // If the file exists, serve it
    if full_path.exists() && full_path.is_file() {
        return serve_file(static_dir, &file_path).await;
    }

    // Check if this looks like a file request (has extension)
    let has_extension = path
        .rsplit('/')
        .next()
        .map(|s| s.contains('.'))
        .unwrap_or(false);

    if has_extension {
        // File request but file not found
        debug!(path = %path, "Static file not found");
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // SPA fallback: serve index.html for routes without extensions
    let index_path = static_dir.join("index.html");
    if index_path.exists() {
        debug!(path = %path, "SPA fallback to index.html");
        return serve_file(static_dir, "index.html").await;
    }

    warn!(static_dir = %static_dir.display(), "index.html not found in static directory");
    (StatusCode::NOT_FOUND, "WebUI not available").into_response()
}

/// Serve a file from the static directory using tower-http's ServeDir.
async fn serve_file(static_dir: &PathBuf, file_path: &str) -> Response {
    let service = ServeDir::new(static_dir);

    let request = Request::builder()
        .uri(format!("/{}", file_path))
        .body(Body::empty())
        .expect("valid request");

    match service.oneshot(request).await {
        Ok(response) => response.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file").into_response(),
    }
}

/// Create a router for serving WebUI static files.
///
/// This should be used as a fallback router after all API routes.
pub fn webui_router(config: WebUiConfig) -> axum::Router {
    let state = WebUiState::new(config.clone());

    if !config.enabled {
        tracing::info!("WebUI serving disabled");
        return axum::Router::new();
    }

    if !config.is_valid() {
        tracing::warn!(
            static_dir = %config.static_dir.display(),
            "WebUI static directory not found or missing index.html"
        );
        return axum::Router::new();
    }

    tracing::info!(
        static_dir = %config.static_dir.display(),
        "WebUI serving enabled"
    );

    // Use ServeDir directly with fallback to index.html for SPA
    let serve_dir =
        ServeDir::new(&config.static_dir).not_found_service(axum::routing::get(move |uri: Uri| {
            let state = state.clone();
            async move { webui_handler(State(state), uri).await }
        }));

    axum::Router::new().fallback_service(serve_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_static_dir() -> TempDir {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create index.html
        fs::write(
            temp_dir.path().join("index.html"),
            "<!DOCTYPE html><html></html>",
        )
        .expect("write index.html");

        // Create _nuxt directory with a JS file
        let nuxt_dir = temp_dir.path().join("_nuxt");
        fs::create_dir(&nuxt_dir).expect("create _nuxt dir");
        fs::write(nuxt_dir.join("app.js"), "console.log('app');").expect("write app.js");

        temp_dir
    }

    #[test]
    fn test_webui_config_default() {
        let config = WebUiConfig::default();
        assert!(config.enabled);
        assert_eq!(
            config.static_dir,
            PathBuf::from("cuttlefish-web/.output/public")
        );
    }

    #[test]
    fn test_webui_config_disabled() {
        let config = WebUiConfig::disabled();
        assert!(!config.enabled);
        assert!(!config.is_valid());
    }

    #[test]
    fn test_webui_config_valid() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        assert!(config.is_valid());
    }

    #[test]
    fn test_webui_config_invalid_no_index() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let config = WebUiConfig::new(temp_dir.path());
        assert!(!config.is_valid());
    }

    #[test]
    fn test_webui_config_invalid_no_dir() {
        let config = WebUiConfig::new("/nonexistent/path");
        assert!(!config.is_valid());
    }

    #[tokio::test]
    async fn test_webui_handler_disabled() {
        let config = WebUiConfig::disabled();
        let state = WebUiState::new(config);

        let uri: Uri = "/".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_webui_handler_api_routes_not_handled() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        let state = WebUiState::new(config);

        // API routes should return 404 (handled by other routers)
        let uri: Uri = "/api/test".parse().expect("valid uri");
        let response = webui_handler(State(state.clone()), uri).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let uri: Uri = "/ws".parse().expect("valid uri");
        let response = webui_handler(State(state.clone()), uri).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let uri: Uri = "/health".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_webui_handler_serves_index() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        let state = WebUiState::new(config);

        let uri: Uri = "/".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webui_handler_serves_static_file() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        let state = WebUiState::new(config);

        let uri: Uri = "/_nuxt/app.js".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webui_handler_spa_fallback() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        let state = WebUiState::new(config);

        // Route without extension should get index.html (SPA fallback)
        let uri: Uri = "/projects/123".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_webui_handler_missing_file_404() {
        let temp_dir = setup_test_static_dir();
        let config = WebUiConfig::new(temp_dir.path());
        let state = WebUiState::new(config);

        // File with extension that doesn't exist should 404
        let uri: Uri = "/missing.js".parse().expect("valid uri");
        let response = webui_handler(State(state), uri).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
