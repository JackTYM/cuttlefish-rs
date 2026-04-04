//! Usage and pricing API endpoints for cost tracking.

use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use cuttlefish_core::{PricingConfig, TimePeriod, UsageStats, pricing::ModelPrice};
use cuttlefish_db::usage::{self, DailyUsage, ProviderUsage, UsageAlert};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::middleware::AuthenticatedUser;

/// Shared state for usage routes.
#[derive(Clone)]
pub struct UsageState {
    /// Database connection pool.
    pub pool: Arc<SqlitePool>,
    /// Usage statistics service.
    pub stats: Arc<UsageStats>,
    /// Pricing configuration (mutable for admin updates).
    pub pricing: Arc<tokio::sync::RwLock<PricingConfig>>,
}

impl UsageState {
    /// Create new usage state.
    pub fn new(pool: Arc<SqlitePool>, pricing: PricingConfig) -> Self {
        let stats = Arc::new(UsageStats::new(pool.clone(), pricing.clone()));
        Self {
            pool,
            stats,
            pricing: Arc::new(tokio::sync::RwLock::new(pricing)),
        }
    }
}

/// Query parameters for usage endpoints.
#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    /// Start date (ISO 8601).
    pub from: Option<String>,
    /// End date (ISO 8601).
    pub to: Option<String>,
    /// Project ID filter.
    pub project_id: Option<String>,
    /// Time period (daily, weekly, monthly).
    pub period: Option<String>,
}

impl UsageQuery {
    fn parse_period(&self) -> TimePeriod {
        match self.period.as_deref() {
            Some("daily") => TimePeriod::Daily,
            Some("weekly") => TimePeriod::Weekly,
            _ => TimePeriod::Monthly,
        }
    }
}

/// Usage summary response.
#[derive(Debug, Serialize)]
pub struct UsageSummaryResponse {
    /// User ID.
    pub user_id: String,
    /// Time period.
    pub period: String,
    /// Total requests.
    pub total_requests: u32,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Cost by provider.
    pub by_provider: std::collections::HashMap<String, f64>,
}

/// Project usage summary response.
#[derive(Debug, Serialize)]
pub struct ProjectUsageResponse {
    /// Project ID.
    pub project_id: String,
    /// Time period.
    pub period: String,
    /// Total requests.
    pub total_requests: u32,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Cost by provider.
    pub by_provider: std::collections::HashMap<String, f64>,
    /// Daily breakdown.
    pub daily_breakdown: Vec<DailyUsage>,
}

/// Daily usage response.
#[derive(Debug, Serialize)]
pub struct DailyUsageResponse {
    /// Daily usage records.
    pub days: Vec<DailyUsage>,
}

/// Provider usage response.
#[derive(Debug, Serialize)]
pub struct ProviderUsageResponse {
    /// Provider usage records.
    pub providers: Vec<ProviderUsage>,
}

/// Pricing table response.
#[derive(Debug, Serialize)]
pub struct PricingResponse {
    /// Pricing by provider and model.
    pub providers:
        std::collections::HashMap<String, std::collections::HashMap<String, ModelPriceResponse>>,
}

/// Model price in response.
#[derive(Debug, Serialize)]
pub struct ModelPriceResponse {
    /// Input price per million tokens.
    pub input_per_million: f64,
    /// Output price per million tokens.
    pub output_per_million: f64,
}

/// Request to update pricing.
#[derive(Debug, Deserialize)]
pub struct UpdatePricingRequest {
    /// Provider name.
    pub provider: String,
    /// Model name.
    pub model: String,
    /// Input price per million tokens.
    pub input_per_million: f64,
    /// Output price per million tokens.
    pub output_per_million: f64,
}

/// Alert response.
#[derive(Debug, Serialize)]
pub struct AlertResponse {
    /// Alert ID.
    pub id: String,
    /// Threshold in USD.
    pub threshold_usd: f64,
    /// Period (daily, weekly, monthly).
    pub period: String,
    /// Project ID (optional).
    pub project_id: Option<String>,
    /// Whether enabled.
    pub enabled: bool,
    /// Last triggered timestamp.
    pub last_triggered_at: Option<String>,
    /// Created timestamp.
    pub created_at: String,
}

impl From<UsageAlert> for AlertResponse {
    fn from(alert: UsageAlert) -> Self {
        Self {
            id: alert.id,
            threshold_usd: alert.threshold_usd,
            period: alert.period,
            project_id: alert.project_id,
            enabled: alert.enabled == 1,
            last_triggered_at: alert.last_triggered_at,
            created_at: alert.created_at,
        }
    }
}

/// Request to create an alert.
#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    /// Threshold in USD.
    pub threshold_usd: f64,
    /// Period (daily, weekly, monthly).
    pub period: String,
    /// Project ID (optional, None = all projects).
    pub project_id: Option<String>,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

fn api_error(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}

/// GET /api/usage - User's usage summary.
pub async fn get_usage_summary(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<UsageQuery>,
) -> ApiResult<UsageSummaryResponse> {
    let period = query.parse_period();

    let summary = state
        .stats
        .user_summary(&user.user_id, period)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(UsageSummaryResponse {
        user_id: summary.user_id,
        period: format!("{:?}", summary.period).to_lowercase(),
        total_requests: summary.total_requests,
        total_input_tokens: summary.total_input_tokens,
        total_output_tokens: summary.total_output_tokens,
        total_cost_usd: summary.total_cost_usd,
        by_provider: summary.by_provider,
    }))
}

/// GET /api/usage/projects/:id - Project usage summary.
pub async fn get_project_usage(
    State(state): State<UsageState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Query(query): Query<UsageQuery>,
) -> ApiResult<ProjectUsageResponse> {
    let period = query.parse_period();

    let summary = state
        .stats
        .project_summary(&project_id, period)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(ProjectUsageResponse {
        project_id: summary.project_id,
        period: format!("{:?}", summary.period).to_lowercase(),
        total_requests: summary.total_requests,
        total_input_tokens: summary.total_input_tokens,
        total_output_tokens: summary.total_output_tokens,
        total_cost_usd: summary.total_cost_usd,
        by_provider: summary.by_provider,
        daily_breakdown: summary.daily_breakdown,
    }))
}

/// GET /api/usage/daily - Daily breakdown.
pub async fn get_daily_usage(
    State(state): State<UsageState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(query): Query<UsageQuery>,
) -> ApiResult<DailyUsageResponse> {
    let period = query.parse_period();

    let days = state
        .stats
        .daily_usage(query.project_id.as_deref(), period)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(DailyUsageResponse { days }))
}

/// GET /api/usage/providers - By-provider breakdown.
pub async fn get_provider_usage(
    State(state): State<UsageState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(query): Query<UsageQuery>,
) -> ApiResult<ProviderUsageResponse> {
    let period = query.parse_period();

    let providers = state
        .stats
        .provider_usage(query.project_id.as_deref(), period)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(ProviderUsageResponse { providers }))
}

/// GET /api/usage/export - Export as CSV.
pub async fn export_usage_csv(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<UsageQuery>,
) -> Result<
    (
        StatusCode,
        [(axum::http::header::HeaderName, &'static str); 2],
        String,
    ),
    (StatusCode, Json<serde_json::Value>),
> {
    let period = query.parse_period();
    let (from, to) = period.range();

    let records = usage::get_usage_by_user(&state.pool, &user.user_id, &from, &to)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let mut csv = String::from(
        "id,project_id,provider,model,input_tokens,output_tokens,request_type,latency_ms,success,created_at\n",
    );

    for r in records {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            r.id,
            r.project_id.unwrap_or_default(),
            r.provider,
            r.model,
            r.input_tokens,
            r.output_tokens,
            r.request_type,
            r.latency_ms.unwrap_or(0),
            r.success,
            r.created_at,
        ));
    }

    Ok((
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/csv"),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "attachment; filename=\"usage.csv\"",
            ),
        ],
        csv,
    ))
}

/// GET /api/pricing - Current pricing table.
pub async fn get_pricing(State(state): State<UsageState>) -> ApiResult<PricingResponse> {
    let pricing = state.pricing.read().await;

    let mut providers = std::collections::HashMap::new();
    for provider_name in pricing.providers() {
        if let Some(models) = pricing.models(provider_name) {
            let mut model_prices = std::collections::HashMap::new();
            for model_name in models {
                if let Some(price) = pricing.get_price(provider_name, model_name) {
                    model_prices.insert(
                        model_name.to_string(),
                        ModelPriceResponse {
                            input_per_million: price.input_per_million,
                            output_per_million: price.output_per_million,
                        },
                    );
                }
            }
            providers.insert(provider_name.to_string(), model_prices);
        }
    }

    Ok(Json(PricingResponse { providers }))
}

/// PUT /api/pricing - Update pricing (admin only).
pub async fn update_pricing(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdatePricingRequest>,
) -> ApiResult<serde_json::Value> {
    if user.user_id != "system" && user.user_id != "admin" {
        return Err(api_error(StatusCode::FORBIDDEN, "Admin access required"));
    }

    let mut pricing = state.pricing.write().await;
    pricing.set_price(
        &req.provider,
        &req.model,
        ModelPrice::new(req.input_per_million, req.output_per_million),
    );

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// GET /api/alerts - List user's alerts.
pub async fn list_alerts(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult<Vec<AlertResponse>> {
    let alerts = usage::get_alerts_by_user(&state.pool, &user.user_id)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(alerts.into_iter().map(AlertResponse::from).collect()))
}

/// POST /api/alerts - Create an alert.
pub async fn create_alert(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateAlertRequest>,
) -> ApiResult<AlertResponse> {
    if !["daily", "weekly", "monthly"].contains(&req.period.as_str()) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Invalid period. Use: daily, weekly, monthly",
        ));
    }

    if req.threshold_usd <= 0.0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Threshold must be positive",
        ));
    }

    let alert = UsageAlert {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: user.user_id.clone(),
        project_id: req.project_id,
        threshold_usd: req.threshold_usd,
        period: req.period,
        last_triggered_at: None,
        enabled: 1,
        created_at: Utc::now().to_rfc3339(),
    };

    usage::create_alert(&state.pool, &alert)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(AlertResponse::from(alert)))
}

/// DELETE /api/alerts/:id - Delete an alert.
pub async fn delete_alert(
    State(state): State<UsageState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(alert_id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let alert = usage::get_alert(&state.pool, &alert_id)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "Alert not found"))?;

    if alert.user_id != user.user_id {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "Cannot delete another user's alert",
        ));
    }

    usage::delete_alert(&state.pool, &alert_id)
        .await
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// Build the usage router.
pub fn usage_router() -> axum::Router<UsageState> {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/api/usage", get(get_usage_summary))
        .route("/api/usage/projects/{id}", get(get_project_usage))
        .route("/api/usage/daily", get(get_daily_usage))
        .route("/api/usage/providers", get(get_provider_usage))
        .route("/api/usage/export", get(export_usage_csv))
        .route("/api/pricing", get(get_pricing))
        .route("/api/pricing", put(update_pricing))
        .route("/api/alerts", get(list_alerts))
        .route("/api/alerts", post(create_alert))
        .route("/api/alerts/{id}", delete(delete_alert))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_query_parse_period() {
        let query = UsageQuery {
            from: None,
            to: None,
            project_id: None,
            period: Some("daily".to_string()),
        };
        assert_eq!(query.parse_period(), TimePeriod::Daily);

        let query = UsageQuery {
            from: None,
            to: None,
            project_id: None,
            period: Some("weekly".to_string()),
        };
        assert_eq!(query.parse_period(), TimePeriod::Weekly);

        let query = UsageQuery {
            from: None,
            to: None,
            project_id: None,
            period: None,
        };
        assert_eq!(query.parse_period(), TimePeriod::Monthly);
    }

    #[test]
    fn test_alert_response_from_usage_alert() {
        let alert = UsageAlert {
            id: "alert-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: Some("proj-1".to_string()),
            threshold_usd: 10.0,
            period: "daily".to_string(),
            last_triggered_at: None,
            enabled: 1,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let response = AlertResponse::from(alert);
        assert_eq!(response.id, "alert-1");
        assert_eq!(response.threshold_usd, 10.0);
        assert!(response.enabled);
    }

    #[test]
    fn test_create_alert_request_deserialize() {
        let json = r#"{"threshold_usd": 50.0, "period": "weekly"}"#;
        let req: CreateAlertRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.threshold_usd, 50.0);
        assert_eq!(req.period, "weekly");
        assert!(req.project_id.is_none());
    }

    #[test]
    fn test_update_pricing_request_deserialize() {
        let json = r#"{"provider": "anthropic", "model": "claude-opus-4-6", "input_per_million": 15.0, "output_per_million": 75.0}"#;
        let req: UpdatePricingRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.provider, "anthropic");
        assert_eq!(req.model, "claude-opus-4-6");
        assert_eq!(req.input_per_million, 15.0);
    }

    #[test]
    fn test_usage_summary_response_serialize() {
        let response = UsageSummaryResponse {
            user_id: "user-1".to_string(),
            period: "monthly".to_string(),
            total_requests: 100,
            total_input_tokens: 50000,
            total_output_tokens: 25000,
            total_cost_usd: 1.25,
            by_provider: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("user-1"));
        assert!(json.contains("1.25"));
    }
}
