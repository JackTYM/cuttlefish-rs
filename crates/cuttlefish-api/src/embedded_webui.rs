//! Embedded WebUI serving for single-binary deployment.
//!
//! Uses rust-embed to include WebUI static files in the binary at compile time.
//! Falls back to file-based serving if embedded assets are not available.

use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};
use rust_embed::Embed;

/// Embedded WebUI assets.
///
/// At compile time, if the `WEBUI_DIR` environment variable is set,
/// rust-embed will include all files from that directory.
/// If WEBUI_DIR is not set, it defaults to an empty placeholder directory.
#[derive(Embed)]
#[folder = "$WEBUI_DIR"]
#[prefix = ""]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.css"]
#[include = "*.json"]
#[include = "*.png"]
#[include = "*.svg"]
#[include = "*.ico"]
#[include = "*.woff"]
#[include = "*.woff2"]
#[include = "*.ttf"]
#[include = "_nuxt/*"]
#[include = "_nuxt/**/*"]
#[include = "builds/*"]
struct EmbeddedAssets;

/// Serve an embedded file by path.
fn serve_embedded(path: &str) -> Response {
    // Try exact path first
    if let Some(content) = EmbeddedAssets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")))
            .header(header::CACHE_CONTROL, "public, max-age=31536000")
            .body(Body::from(content.data.into_owned()))
            .unwrap_or_else(|_| Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal error"))
                .expect("valid response"));
    }

    // Not found
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found"))
        .unwrap_or_else(|_| Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .expect("valid response"))
}

/// Handler for embedded WebUI requests.
async fn embedded_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Serve exact file if it exists
    if !path.is_empty() && EmbeddedAssets::get(path).is_some() {
        return serve_embedded(path);
    }

    // For paths without extensions (SPA routes), serve index.html
    let has_extension = path.rsplit('/').next().is_some_and(|s| s.contains('.'));
    let should_serve_index = !has_extension || path.is_empty();

    if should_serve_index && let Some(content) = EmbeddedAssets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(content.data.into_owned()))
            .unwrap_or_else(|_| Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal error"))
                .expect("valid response"));
    }

    // File not found
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found"))
        .unwrap_or_else(|_| Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .expect("valid response"))
}

/// Check if embedded WebUI assets are available.
pub fn has_embedded_webui() -> bool {
    EmbeddedAssets::get("index.html").is_some()
}

/// Create a router for embedded WebUI.
///
/// Returns None if no embedded assets are available.
pub fn embedded_webui_router() -> Option<Router> {
    if !has_embedded_webui() {
        tracing::info!("No embedded WebUI found, will use file-based serving if configured");
        return None;
    }

    let file_count = EmbeddedAssets::iter().count();
    tracing::info!(
        files = file_count,
        "Serving embedded WebUI ({} files)",
        file_count
    );

    Some(Router::new().fallback(get(embedded_handler)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_embedded_webui() {
        // This will depend on whether WEBUI_DIR was set at compile time
        let _ = has_embedded_webui();
    }

    #[tokio::test]
    async fn test_embedded_handler_returns_response() {
        let uri: Uri = "/".parse().expect("valid uri");
        let response = embedded_handler(uri).await;
        // Will be 404 if no assets embedded, OK if they are
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
        );
    }
}
