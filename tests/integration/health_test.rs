//! Integration test for health endpoint.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use cuttlefish_agents::TokioMessageBus;
use cuttlefish_api::{build_app, routes::AppState};
use cuttlefish_core::TemplateRegistry;
use cuttlefish_db::Database;
use cuttlefish_providers::ProviderRegistry;
use dashmap::DashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

async fn create_test_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let db = Database::open(&db_path).await.expect("open database");

    let state = AppState {
        api_key: "test-api-key".to_string(),
        template_registry: Arc::new(TemplateRegistry::new()),
        db: Arc::new(db),
        provider_registry: Arc::new(ProviderRegistry::new()),
        active_sessions: Arc::new(DashMap::new()),
        message_bus: Arc::new(TokioMessageBus::new()),
        prompts_dir: temp_dir.path().to_path_buf(),
        default_provider: None,
        approval_registry: cuttlefish_api::create_approval_registry(),
    };

    (state, temp_dir)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("parse json");

    assert_eq!(json["status"], "ok");
    assert!(json["version"].as_str().is_some());
}

#[tokio::test]
async fn test_not_found_returns_404() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let request = Request::builder()
        .uri("/nonexistent")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
