# V1 Cost Tracking System — BYOK Usage Monitoring & Reporting

## TL;DR

> **Quick Summary**: Build a comprehensive cost tracking system that logs all API token usage per provider, calculates costs using configurable pricing, aggregates by project/session/user, and displays usage dashboards for BYOK (Bring Your Own Key) transparency.
> 
> **Deliverables**:
> - Token usage logging for all provider requests
> - Configurable pricing per model (input/output rates)
> - Per-project, per-session, per-user cost aggregation
> - Usage dashboard in WebUI with charts
> - API endpoints for usage data
> - Daily/weekly/monthly cost reports
> - Optional spending alerts
> 
> **Estimated Effort**: Medium-Large (3-4 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Schema) → Tasks 2-4 (Logging + Pricing) → Tasks 5-7 (Aggregation + API) → Tasks 8-10 (Dashboard + Alerts)

---

## Context

### Original Request (From Product Spec)
- **BYOK Model**: Users bring their own API keys for model providers
- **Cost Transparency**: Users need to know how much they're spending per project
- **Usage Tracking**: Track tokens, requests, and costs across providers

### Problem Statement
Users using their own API keys have no visibility into:
- How many tokens each project consumes
- Which models are most expensive for their workload
- Total spending across all projects
- Cost trends over time

Without tracking, users can be surprised by bills from Anthropic, OpenAI, etc.

### Design Philosophy
- **Non-blocking**: Logging should not slow down API requests
- **Accurate**: Token counts from provider responses (not estimates)
- **Configurable**: Users can set their own pricing (accounts differ)
- **Aggregatable**: Easy to roll up by project, session, user, provider, time period

### Existing Infrastructure
**Already implemented:**
- `CompletionResponse` includes `input_tokens` and `output_tokens`
- `StreamChunk::Usage` provides tokens at end of streams
- `Conversation` model has `token_count` field
- Provider trait supports `count_tokens()` method

**Missing:**
- No dedicated usage logging table
- No pricing configuration
- No aggregation queries
- No dashboard UI
- No spending alerts

---

## Work Objectives

### Core Objective
Track all API token usage, calculate costs using configurable pricing, and provide transparency dashboards so BYOK users understand their spending.

### Concrete Deliverables
- `crates/cuttlefish-db/src/usage.rs` — Usage logging and queries
- `crates/cuttlefish-core/src/pricing.rs` — Pricing configuration
- `crates/cuttlefish-api/src/usage_routes.rs` — Usage API endpoints
- Database migrations for usage tracking tables
- WebUI usage dashboard with charts
- CLI commands: `cuttlefish usage`, `cuttlefish costs`

### Definition of Done
- [ ] All API requests log token usage to database
- [ ] Costs calculated using configurable pricing
- [ ] Usage queryable by project, session, user, provider, time
- [ ] Dashboard shows usage charts
- [ ] `cargo test --workspace usage` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- Token usage logging (input + output) per request
- Request metadata: provider, model, project_id, session_id, user_id, timestamp
- Configurable pricing per model (USD per 1M tokens, input vs output rates)
- Aggregation queries: by project, by session, by user, by provider, by day/week/month
- REST API for usage data
- WebUI dashboard with usage breakdown
- CLI to view usage/costs

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No hardcoded pricing (all configurable)
- No blocking on usage logging (async fire-and-forget)
- No PII in usage logs (no message content)
- No external analytics services (self-hosted only)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (database, providers)
- **Automated tests**: YES (TDD)
- **Framework**: `#[tokio::test]` for async

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — schema + logging):
├── Task 1: Usage tracking database schema [deep]
├── Task 2: Usage logging middleware [deep]
└── Task 3: Pricing configuration [unspecified-high]

Wave 2 (Aggregation — queries + calculations):
├── Task 4: Cost calculation engine [unspecified-high]
├── Task 5: Aggregation queries [deep]
└── Task 6: Usage statistics service [unspecified-high]

Wave 3 (API + CLI):
├── Task 7: Usage API endpoints [unspecified-high]
├── Task 8: CLI usage commands [quick]
└── Task 9: Spending alerts [unspecified-high]

Wave 4 (Dashboard):
└── Task 10: WebUI usage dashboard [visual-engineering]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Usage tracking E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 4 → Task 5 → Task 7 → Task 10 → F1-F4 → user okay
Parallel Speedup: ~50% faster than sequential
Max Concurrent: 3 (Waves 1-3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 4, 5 | 1 |
| 2 | 1 | 4, 6 | 1 |
| 3 | — | 4 | 1 |
| 4 | 1, 2, 3 | 5, 6, 7 | 2 |
| 5 | 1, 4 | 6, 7, 10 | 2 |
| 6 | 2, 4, 5 | 7, 10 | 2 |
| 7 | 4, 5, 6 | 10 | 3 |
| 8 | 5 | F1-F4 | 3 |
| 9 | 5, 6 | F1-F4 | 3 |
| 10 | 5, 6, 7 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1-T2 → `deep`, T3 → `unspecified-high`
- **Wave 2**: 3 tasks — T4, T6 → `unspecified-high`, T5 → `deep`
- **Wave 3**: 3 tasks — T7, T9 → `unspecified-high`, T8 → `quick`
- **Wave 4**: 1 task — T10 → `visual-engineering`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Usage Tracking Database Schema

  **What to do**:
  - Create database migration for usage tables:
    ```sql
    CREATE TABLE api_usage (
        id TEXT PRIMARY KEY,
        project_id TEXT REFERENCES projects(id),
        session_id TEXT,
        user_id TEXT,
        provider TEXT NOT NULL,
        model TEXT NOT NULL,
        input_tokens INTEGER NOT NULL,
        output_tokens INTEGER NOT NULL,
        request_type TEXT NOT NULL,  -- 'complete', 'stream'
        latency_ms INTEGER,
        success INTEGER NOT NULL DEFAULT 1,
        error_type TEXT,
        created_at TEXT NOT NULL
    );
    CREATE INDEX idx_usage_project ON api_usage(project_id, created_at DESC);
    CREATE INDEX idx_usage_user ON api_usage(user_id, created_at DESC);
    CREATE INDEX idx_usage_provider ON api_usage(provider, created_at DESC);
    CREATE INDEX idx_usage_date ON api_usage(created_at);
    
    CREATE TABLE model_pricing (
        id TEXT PRIMARY KEY,
        provider TEXT NOT NULL,
        model TEXT NOT NULL,
        input_price_per_million REAL NOT NULL,  -- USD per 1M tokens
        output_price_per_million REAL NOT NULL,
        effective_from TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE(provider, model, effective_from)
    );
    CREATE INDEX idx_pricing_lookup ON model_pricing(provider, model);
    
    CREATE TABLE usage_alerts (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        project_id TEXT,  -- NULL = all projects
        threshold_usd REAL NOT NULL,
        period TEXT NOT NULL,  -- 'daily', 'weekly', 'monthly'
        last_triggered_at TEXT,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL
    );
    CREATE INDEX idx_alerts_user ON usage_alerts(user_id);
    ```
  - Add CRUD operations to `crates/cuttlefish-db/src/usage.rs`
  - Create `ApiUsage`, `ModelPricing`, `UsageAlert` structs

  **Must NOT do**:
  - Don't store message content (privacy)
  - Don't use REAL for tokens (use INTEGER)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1)
  - **Blocks**: Tasks 2, 4, 5
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-db/src/lib.rs` — Migration pattern
  - `crates/cuttlefish-db/src/models.rs` — Model patterns

  **Acceptance Criteria**:
  - [ ] Tables created on migration
  - [ ] CRUD operations work
  - [ ] Indexes created

  **QA Scenarios**:
  ```
  Scenario: Usage record inserted and retrieved
    Tool: Bash (cargo test)
    Steps:
      1. Insert usage record for project
      2. Query by project_id
      3. Verify all fields match
    Expected Result: Round-trip works
    Evidence: .sisyphus/evidence/task-1-usage-crud.txt
  ```

  **Commit**: YES
  - Message: `feat(db): add usage tracking schema`
  - Files: `db/usage.rs`, migrations

- [ ] 2. Usage Logging Middleware

  **What to do**:
  - Create provider wrapper that logs usage:
    ```rust
    pub struct TrackedProvider<P: ModelProvider> {
        inner: P,
        db: Database,
        context: UsageContext,  // project_id, session_id, user_id
    }
    
    impl<P: ModelProvider> ModelProvider for TrackedProvider<P> {
        async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
            let start = Instant::now();
            let result = self.inner.complete(request).await;
            let latency = start.elapsed().as_millis() as i64;
            
            // Log asynchronously (fire-and-forget)
            self.log_usage(&result, latency).await;
            
            result
        }
        // Similar for stream()
    }
    ```
  - Capture: provider, model, input_tokens, output_tokens, latency, success/error
  - Non-blocking logging via tokio::spawn
  - Handle stream completion (log when Usage chunk arrives)

  **Must NOT do**:
  - Don't block on logging (fire-and-forget)
  - Don't log request content

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Tasks 4, 6
  - **Blocked By**: Task 1

  **References**:
  - `crates/cuttlefish-core/src/traits/provider.rs` — ModelProvider trait
  - `crates/cuttlefish-providers/src/bedrock.rs` — Provider pattern

  **Acceptance Criteria**:
  - [ ] Usage logged for complete() calls
  - [ ] Usage logged for stream() calls (at end)
  - [ ] Logging doesn't block requests
  - [ ] Errors logged with error_type

  **QA Scenarios**:
  ```
  Scenario: Provider logs usage on completion
    Tool: Bash (cargo test)
    Steps:
      1. Create TrackedProvider with mock inner
      2. Call complete()
      3. Query database for usage record
      4. Verify tokens match response
    Expected Result: Usage logged correctly
    Evidence: .sisyphus/evidence/task-2-logging.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. Pricing Configuration

  **What to do**:
  - Create `crates/cuttlefish-core/src/pricing.rs`:
    ```rust
    pub struct PricingConfig {
        pub provider: String,
        pub model: String,
        pub input_price_per_million: f64,
        pub output_price_per_million: f64,
    }
    
    impl PricingConfig {
        pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_price_per_million;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_price_per_million;
            input_cost + output_cost
        }
    }
    ```
  - Default pricing table (can be overridden in config):
    ```toml
    [pricing.anthropic]
    "claude-opus-4-6" = { input = 15.0, output = 75.0 }
    "claude-sonnet-4-6" = { input = 3.0, output = 15.0 }
    "claude-haiku-4-5" = { input = 0.25, output = 1.25 }
    
    [pricing.openai]
    "gpt-5.4" = { input = 10.0, output = 30.0 }
    "gpt-4o" = { input = 2.5, output = 10.0 }
    ```
  - Load from config, seed database, allow per-user overrides

  **Must NOT do**:
  - Don't hardcode prices in code
  - Don't assume prices are static (they change)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Task 4
  - **Blocked By**: None

  **References**:
  - Anthropic pricing: https://www.anthropic.com/pricing
  - OpenAI pricing: https://openai.com/pricing

  **Acceptance Criteria**:
  - [ ] Default pricing loaded from config
  - [ ] Pricing stored in database
  - [ ] Cost calculation correct

  **QA Scenarios**:
  ```
  Scenario: Cost calculation accuracy
    Tool: Bash (cargo test)
    Steps:
      1. Set pricing: $15/M input, $75/M output
      2. Calculate cost for 1000 input, 500 output tokens
      3. Expected: (1000/1M)*15 + (500/1M)*75 = $0.015 + $0.0375 = $0.0525
      4. Verify calculation matches
    Expected Result: $0.0525
    Evidence: .sisyphus/evidence/task-3-pricing.txt
  ```

  **Commit**: YES (Wave 1)
  - Message: `feat(core): add usage logging and pricing configuration`
  - Files: `core/pricing.rs`, `db/usage.rs`

- [ ] 4. Cost Calculation Engine

  **What to do**:
  - Create service to calculate costs from usage:
    ```rust
    pub struct CostCalculator {
        db: Database,
    }
    
    impl CostCalculator {
        pub async fn calculate_request_cost(&self, usage: &ApiUsage) -> Result<f64, CostError> {
            let pricing = self.get_pricing(&usage.provider, &usage.model).await?;
            Ok(pricing.calculate_cost(usage.input_tokens, usage.output_tokens))
        }
        
        pub async fn calculate_total_cost(
            &self,
            usages: &[ApiUsage],
        ) -> Result<CostBreakdown, CostError> {
            // Returns breakdown by provider/model
        }
    }
    
    pub struct CostBreakdown {
        pub total_usd: f64,
        pub by_provider: HashMap<String, f64>,
        pub by_model: HashMap<String, f64>,
        pub input_tokens_total: u64,
        pub output_tokens_total: u64,
    }
    ```
  - Handle missing pricing (warn, use default)
  - Support historical pricing (use effective_from date)

  **Must NOT do**:
  - Don't fail silently if pricing missing (warn)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 2)
  - **Blocks**: Tasks 5, 6, 7
  - **Blocked By**: Tasks 1, 2, 3

  **Acceptance Criteria**:
  - [ ] Request costs calculated correctly
  - [ ] Breakdown by provider/model works
  - [ ] Missing pricing handled gracefully

  **QA Scenarios**:
  ```
  Scenario: Calculate breakdown for multiple requests
    Tool: Bash (cargo test)
    Steps:
      1. Insert 10 usage records (5 Anthropic, 5 OpenAI)
      2. Calculate total cost
      3. Verify by_provider sums match
    Expected Result: Correct breakdown
    Evidence: .sisyphus/evidence/task-4-breakdown.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Aggregation Queries

  **What to do**:
  - Implement efficient aggregation queries:
    ```rust
    pub struct UsageQuery {
        db: Database,
    }
    
    impl UsageQuery {
        pub async fn by_project(
            &self,
            project_id: &str,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<ApiUsage>, DbError>
        
        pub async fn by_user(
            &self,
            user_id: &str,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<ApiUsage>, DbError>
        
        pub async fn aggregated_by_day(
            &self,
            project_id: Option<&str>,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<DailyUsage>, DbError>
        
        pub async fn aggregated_by_provider(
            &self,
            project_id: Option<&str>,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<ProviderUsage>, DbError>
        
        pub async fn top_projects_by_cost(
            &self,
            user_id: &str,
            limit: usize,
        ) -> Result<Vec<ProjectCost>, DbError>
    }
    
    pub struct DailyUsage {
        pub date: String,
        pub input_tokens: u64,
        pub output_tokens: u64,
        pub request_count: u32,
        pub estimated_cost: f64,
    }
    ```
  - Use SQLite GROUP BY with date truncation
  - Support pagination for large datasets

  **Must NOT do**:
  - Don't load all records into memory (aggregate in SQL)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Blocks**: Tasks 6, 7, 10
  - **Blocked By**: Tasks 1, 4

  **Acceptance Criteria**:
  - [ ] By-project queries work
  - [ ] By-user queries work
  - [ ] Daily aggregation correct
  - [ ] Top projects ranked correctly

  **QA Scenarios**:
  ```
  Scenario: Daily aggregation sums correctly
    Tool: Bash (cargo test)
    Steps:
      1. Insert 100 records across 7 days
      2. Query aggregated_by_day
      3. Verify each day's sum matches manual calculation
    Expected Result: Daily totals accurate
    Evidence: .sisyphus/evidence/task-5-aggregation.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Usage Statistics Service

  **What to do**:
  - Create high-level service combining queries and costs:
    ```rust
    pub struct UsageStats {
        query: UsageQuery,
        calculator: CostCalculator,
    }
    
    impl UsageStats {
        pub async fn project_summary(
            &self,
            project_id: &str,
            period: TimePeriod,
        ) -> Result<ProjectUsageSummary, Error>
        
        pub async fn user_summary(
            &self,
            user_id: &str,
            period: TimePeriod,
        ) -> Result<UserUsageSummary, Error>
        
        pub async fn current_period_cost(
            &self,
            user_id: &str,
            period: TimePeriod,
        ) -> Result<f64, Error>
    }
    
    pub struct ProjectUsageSummary {
        pub project_id: String,
        pub project_name: String,
        pub period: TimePeriod,
        pub total_requests: u32,
        pub total_input_tokens: u64,
        pub total_output_tokens: u64,
        pub total_cost_usd: f64,
        pub by_provider: HashMap<String, ProviderUsage>,
        pub daily_breakdown: Vec<DailyUsage>,
    }
    ```

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Blocks**: Tasks 7, 10
  - **Blocked By**: Tasks 2, 4, 5

  **Acceptance Criteria**:
  - [ ] Project summary includes all data
  - [ ] User summary aggregates across projects
  - [ ] Time periods handled correctly

  **QA Scenarios**:
  ```
  Scenario: Project summary accuracy
    Tool: Bash (cargo test)
    Steps:
      1. Create project with known usage
      2. Get project summary for week
      3. Verify all totals match
    Expected Result: Summary accurate
    Evidence: .sisyphus/evidence/task-6-summary.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(core): add cost calculation and usage aggregation`
  - Files: `core/costs.rs`, `db/usage.rs`

- [ ] 7. Usage API Endpoints

  **What to do**:
  - Create `crates/cuttlefish-api/src/usage_routes.rs`:
    - `GET /api/usage` — User's usage summary (current period)
    - `GET /api/usage/projects/:id` — Project usage summary
    - `GET /api/usage/daily` — Daily breakdown
    - `GET /api/usage/providers` — By-provider breakdown
    - `GET /api/usage/export` — Export as CSV
    - `GET /api/pricing` — Current pricing table
    - `PUT /api/pricing` — Update pricing (admin)
    - `GET /api/alerts` — User's spending alerts
    - `POST /api/alerts` — Create alert
    - `DELETE /api/alerts/:id` — Delete alert
  - Query parameters: `from`, `to`, `project_id`, `period`
  - Pagination for lists
  - CSV export for accounting

  **Must NOT do**:
  - Don't expose other users' usage data
  - Don't allow non-admins to modify global pricing

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 3)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 4, 5, 6

  **References**:
  - `crates/cuttlefish-api/src/routes.rs` — Route patterns

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] Permissions enforced
  - [ ] CSV export valid

  **QA Scenarios**:
  ```
  Scenario: Get usage summary via API
    Tool: Bash (curl)
    Steps:
      1. GET /api/usage with auth header
      2. Verify JSON response contains totals
      3. GET /api/usage/daily
      4. Verify daily breakdown
    Expected Result: API returns correct data
    Evidence: .sisyphus/evidence/task-7-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add usage and pricing endpoints`
  - Files: `api/usage_routes.rs`

- [ ] 8. CLI Usage Commands

  **What to do**:
  - Add commands to main binary:
    - `cuttlefish usage` — Current period summary
    - `cuttlefish usage --project <id>` — Project usage
    - `cuttlefish usage --daily` — Daily breakdown table
    - `cuttlefish usage --export <path>` — Export CSV
    - `cuttlefish costs` — Alias for usage with cost focus
    - `cuttlefish pricing` — Show pricing table
    - `cuttlefish pricing set <provider> <model> <input> <output>` — Set pricing
  - Human-readable tables with colors
  - `--json` flag for machine output

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 5

  **Acceptance Criteria**:
  - [ ] Commands display correctly
  - [ ] Tables formatted nicely
  - [ ] JSON output valid

  **QA Scenarios**:
  ```
  Scenario: CLI usage display
    Tool: Bash
    Steps:
      1. Run `cuttlefish usage`
      2. Verify table shows totals
      3. Run `cuttlefish usage --json`
      4. Verify valid JSON
    Expected Result: CLI works
    Evidence: .sisyphus/evidence/task-8-cli.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Spending Alerts

  **What to do**:
  - Implement alert system:
    ```rust
    pub struct AlertChecker {
        db: Database,
        stats: UsageStats,
        notifier: Box<dyn AlertNotifier>,
    }
    
    impl AlertChecker {
        pub async fn check_alerts(&self) -> Result<Vec<TriggeredAlert>, Error> {
            for alert in self.get_active_alerts().await? {
                let current_cost = self.stats.current_period_cost(
                    &alert.user_id,
                    alert.period.into(),
                ).await?;
                
                if current_cost >= alert.threshold_usd {
                    self.trigger_alert(&alert, current_cost).await?;
                }
            }
        }
    }
    
    pub trait AlertNotifier: Send + Sync {
        fn notify(&self, alert: &UsageAlert, current_cost: f64) -> Result<(), Error>;
    }
    ```
  - Notification channels: WebSocket push, Discord (if configured)
  - Run check periodically (every hour)
  - Don't spam (track last_triggered_at)

  **Must NOT do**:
  - Don't send email (not implemented in V1)
  - Don't trigger same alert repeatedly in same period

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 5, 6

  **Acceptance Criteria**:
  - [ ] Alerts trigger at threshold
  - [ ] Notifications sent
  - [ ] No duplicate triggers

  **QA Scenarios**:
  ```
  Scenario: Alert triggers at threshold
    Tool: Bash (cargo test)
    Steps:
      1. Create alert: $10 daily threshold
      2. Insert usage totaling $11
      3. Run alert check
      4. Verify alert triggered
      5. Run again, verify no duplicate
    Expected Result: Single trigger
    Evidence: .sisyphus/evidence/task-9-alerts.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(core): add spending alerts and CLI usage commands`
  - Files: `core/alerts.rs`, CLI commands

- [ ] 10. WebUI Usage Dashboard

  **What to do**:
  - Create `cuttlefish-web/pages/usage.vue`:
    - Period selector (day, week, month, custom)
    - Summary cards: Total cost, Total tokens, Request count
    - Line chart: Daily costs over time (Chart.js or similar)
    - Bar chart: Costs by provider
    - Table: Top projects by cost
    - Breakdown by model
  - Create `cuttlefish-web/components/UsageChart.vue`
  - Create `cuttlefish-web/components/CostCard.vue`
  - Add "Usage" link to sidebar navigation
  - Mobile responsive design

  **Must NOT do**:
  - Don't add heavy charting library (use lightweight option)
  - Don't show other users' data

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 5, 6, 7

  **References**:
  - `cuttlefish-web/pages/index.vue` — Page patterns
  - Terminal/hacker aesthetic from v1-webui.md

  **Acceptance Criteria**:
  - [ ] Dashboard renders without errors
  - [ ] Charts display correct data
  - [ ] Period selection works
  - [ ] Mobile responsive

  **QA Scenarios**:
  ```
  Scenario: Usage dashboard displays data
    Tool: Playwright
    Steps:
      1. Navigate to /usage
      2. Verify summary cards show values
      3. Select "Week" period
      4. Verify chart updates
      5. Check table shows top projects
    Expected Result: Dashboard functional
    Evidence: .sisyphus/evidence/task-10-dashboard.png
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(web): add usage dashboard with charts`
  - Files: `pages/usage.vue`, `components/UsageChart.vue`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify usage logging, pricing, aggregation, API, CLI, dashboard, alerts all implemented.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + tests. Review for no PII in logs, proper error handling.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Usage Tracking E2E QA** — `unspecified-high`
  Full workflow: Make API requests, verify usage logged, check costs calculated, view dashboard, trigger alert.
  Output: `Logging [PASS/FAIL] | Costs [PASS/FAIL] | Dashboard [PASS/FAIL] | Alerts [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no message content in logs, pricing configurable (not hardcoded), logging non-blocking.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(db): add usage tracking schema` |
| 1 | `feat(core): add usage logging and pricing configuration` |
| 2 | `feat(core): add cost calculation and usage aggregation` |
| 3 | `feat(api): add usage and pricing endpoints` |
| 3 | `feat(core): add spending alerts and CLI usage commands` |
| 4 | `feat(web): add usage dashboard with charts` |

---

## Success Criteria

### Verification Commands
```bash
cargo test --workspace usage  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
curl localhost:8080/api/usage  # Returns usage summary
cuttlefish usage  # Shows usage table
```

### Final Checklist
- [ ] All API requests log token usage
- [ ] Costs calculated using configurable pricing
- [ ] Aggregation by project/session/user/provider works
- [ ] Dashboard shows usage charts
- [ ] Alerts trigger at threshold
- [ ] No message content in usage logs
- [ ] Logging doesn't block API requests
