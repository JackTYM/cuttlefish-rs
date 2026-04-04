# V1 Cost Tracking System — Work Breakdown Structure

## Overview
- **Plan**: `.sisyphus/plans/v1-costs.md`
- **Total Tasks**: 10 + 4 verification = 14 task groups
- **Estimated Duration**: 3-4 days
- **Parallel Execution**: YES - 4 internal waves

## Quick Reference

| Wave | Tasks | Category | Parallel |
|------|-------|----------|----------|
| 1 | T1: Usage schema, T2: Logging middleware, T3: Pricing config | deep, unspecified-high | YES |
| 2 | T4: Cost calculator, T5: Aggregation queries, T6: Stats service | deep, unspecified-high | YES |
| 3 | T7: Usage API, T8: CLI commands, T9: Spending alerts | unspecified-high, quick | YES |
| 4 | T10: WebUI dashboard | visual-engineering | NO |
| FINAL | F1-F4: Verification | oracle, deep | YES |

## Task Summary

### Wave 1: Foundation
- **T1**: Create `api_usage`, `model_pricing`, `usage_alerts` tables
- **T2**: Create `TrackedProvider<P>` wrapper that logs usage async
- **T3**: Create `pricing.rs` with default pricing table (Claude, OpenAI, etc.)

### Wave 2: Aggregation
- **T4**: `CostCalculator` service with per-request and breakdown calculations
- **T5**: SQL aggregation queries by project/user/day/provider
- **T6**: `UsageStats` high-level service for summaries

### Wave 3: API & CLI
- **T7**: 10 usage/pricing/alerts endpoints
- **T8**: `cuttlefish usage`, `cuttlefish costs`, `cuttlefish pricing` commands
- **T9**: `AlertChecker` with threshold triggers and notifications

### Wave 4: Dashboard
- **T10**: Vue usage dashboard with Chart.js charts, cost cards, breakdown tables

## Dependencies
```
T1 → T2 → T4 → T5 → T6 → T7 → T10
T3 → T4
T5 → T8, T9
```

## Files to Create/Modify
- `crates/cuttlefish-db/src/usage.rs`
- `crates/cuttlefish-core/src/pricing.rs`
- `crates/cuttlefish-core/src/costs.rs`
- `crates/cuttlefish-core/src/alerts.rs`
- `crates/cuttlefish-providers/src/tracked.rs`
- `crates/cuttlefish-api/src/usage_routes.rs`
- `cuttlefish-web/pages/usage.vue`
- `cuttlefish-web/components/UsageChart.vue`
- `cuttlefish-web/components/CostCard.vue`

## Pricing Defaults (USD per 1M tokens)
```toml
[pricing.anthropic]
claude-opus-4-6 = { input = 15.0, output = 75.0 }
claude-sonnet-4-6 = { input = 3.0, output = 15.0 }
claude-haiku-4-5 = { input = 0.25, output = 1.25 }

[pricing.openai]
gpt-5.4 = { input = 10.0, output = 30.0 }
```

## Success Criteria
- All API requests log token usage
- Costs calculated with configurable pricing
- Aggregation by project/user/provider/day works
- Dashboard shows usage charts
- Spending alerts trigger at threshold
