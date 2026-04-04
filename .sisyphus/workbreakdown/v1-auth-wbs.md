# V1 Auth System — Work Breakdown Structure

## Overview
- **Plan**: `.sisyphus/plans/v1-auth.md`
- **Total Tasks**: 12 + 4 verification = 16 task groups
- **Estimated Duration**: 4-5 days
- **Parallel Execution**: YES - 4 internal waves

## Quick Reference

| Wave | Tasks | Category | Parallel |
|------|-------|----------|----------|
| 1 | T1: User model, T2: Password (Argon2), T3: JWT | deep, unspecified-high | YES |
| 2 | T4: Sessions, T5: Refresh tokens, T6: Auth middleware | deep, unspecified-high | YES |
| 3 | T7: API keys, T8: Roles, T9: Password reset | unspecified-high, deep | YES |
| 4 | T10: API routes, T11: WebUI pages, T12: Backwards compat | visual-engineering, quick | YES |
| FINAL | F1-F4: Verification | oracle, deep | YES |

## Task Summary

### Wave 1: Foundation
- **T1**: Create `users` table, `User` struct, CRUD in `cuttlefish-db/src/auth.rs`
- **T2**: Add `argon2` crate, create `password.rs` with hash/verify functions
- **T3**: Use `jsonwebtoken` crate, create `jwt.rs` with access/refresh tokens

### Wave 2: Sessions
- **T4**: Create `sessions` table, session management functions
- **T5**: Implement refresh token rotation with reuse detection
- **T6**: Create `cuttlefish-api/src/middleware/auth.rs` with JWT + API key support

### Wave 3: Keys & Roles
- **T7**: Create `api_keys` table, key generation with `cfish_` prefix
- **T8**: Create `project_members` table, role enum (owner/admin/member/viewer)
- **T9**: Password reset tokens with 1-hour expiry

### Wave 4: Integration
- **T10**: 13 auth endpoints in `cuttlefish-api/src/auth_routes.rs`
- **T11**: Login/register/reset Vue pages with `useAuth()` composable
- **T12**: Support legacy `CUTTLEFISH_API_KEY` env var

## Dependencies
```
T1 → T2 → T4 → T6 → T10 → T11
T1 → T3 → T4
T1 → T7 → T10
T1 → T8 → T10
T1 → T2 → T9 → T10
```

## Files to Create/Modify
- `crates/cuttlefish-core/src/auth/user.rs`
- `crates/cuttlefish-core/src/auth/password.rs`
- `crates/cuttlefish-core/src/auth/jwt.rs`
- `crates/cuttlefish-db/src/auth.rs`
- `crates/cuttlefish-api/src/auth_routes.rs`
- `crates/cuttlefish-api/src/middleware/auth.rs`
- `cuttlefish-web/pages/login.vue`
- `cuttlefish-web/pages/register.vue`
- `cuttlefish-web/pages/reset-password.vue`

## Success Criteria
- Users can register/login with email/password
- JWT tokens with 24h access, 30d refresh
- API keys with `cfish_` prefix and scopes
- Role-based project access control
- All tests pass, clippy clean
