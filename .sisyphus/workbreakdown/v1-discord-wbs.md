# V1 Discord Bot — Work Breakdown Structure

## Overview
- **Plan**: `.sisyphus/plans/v1-discord.md`
- **Total Tasks**: 13 + 4 verification = 17 task groups
- **Estimated Duration**: 4-5 days
- **Parallel Execution**: YES - 5 internal waves

## Quick Reference

| Wave | Tasks | Category | Parallel |
|------|-------|----------|----------|
| 1 | T1: Command framework, T2: /new-project, T3: /status | quick, unspecified-high | YES |
| 2 | T4: /logs, T5: /approve+/reject, T6: Channel auto-create | quick, deep | YES |
| 3 | T7: Notification system, T8: Pending action detection, T9: User routing | deep, quick | YES |
| 4 | T10: Agent embeds, T11: Progress embeds, T12: Channel archival | visual-engineering, unspecified-high | YES |
| 5 | T13: Wire to Cuttlefish API | deep | NO |
| FINAL | F1-F4: Verification | oracle, deep | YES |

## Task Summary

### Wave 1: Commands
- **T1**: Slash command framework with serenity's `CreateCommand` builder
- **T2**: `/new-project <name> [template]` — create project + channel
- **T3**: `/status [project]` — show project/agent status embed

### Wave 2: Core Features
- **T4**: `/logs [project] [lines]` — formatted activity logs with pagination
- **T5**: `/approve` and `/reject [reason]` — handle pending actions
- **T6**: Auto-create `project-{name}` channels in designated category

### Wave 3: Notifications
- **T7**: `NotificationManager` with urgency levels (info, action_required, error)
- **T8**: `PendingAction` detection from agent output
- **T9**: User mention routing (owner → last active → admin fallback)

### Wave 4: Embeds
- **T10**: Rich agent status embeds with emojis and color coding
- **T11**: Progress indicator embeds with block character progress bars
- **T12**: Channel archival (move to archive category, read-only)

### Wave 5: Integration
- **T13**: HTTP client for Cuttlefish API, wire all commands

## Dependencies
```
T1 → T2 → T6 → T7 → T9 → T13
T1 → T3, T4, T5 → T7
T5 → T8 → T9
T10 → T11 → T13
T6 → T12 → T13
```

## Files to Create/Modify
- `crates/cuttlefish-discord/src/commands/mod.rs`
- `crates/cuttlefish-discord/src/commands/new_project.rs`
- `crates/cuttlefish-discord/src/commands/status.rs`
- `crates/cuttlefish-discord/src/commands/logs.rs`
- `crates/cuttlefish-discord/src/commands/approve.rs`
- `crates/cuttlefish-discord/src/commands/reject.rs`
- `crates/cuttlefish-discord/src/notifications.rs`
- `crates/cuttlefish-discord/src/embeds.rs`
- `crates/cuttlefish-discord/src/api_client.rs`
- `crates/cuttlefish-discord/src/channel_manager.rs` (enhance)

## Slash Commands
| Command | Options | Description |
|---------|---------|-------------|
| `/new-project` | name, template?, description? | Create project + channel |
| `/status` | project? | Show project/agent status |
| `/logs` | project?, lines? | Recent activity logs |
| `/approve` | — | Approve pending action |
| `/reject` | reason? | Reject with feedback |
| `/unarchive` | project | Restore archived channel |

## Embed Colors
- 🟢 Green: Success/Complete
- 🟡 Yellow: Working/In Progress
- 🔴 Red: Error/Failed
- ⚫ Gray: Waiting/Idle

## Success Criteria
- All 5+ slash commands work in Discord
- Project channels auto-created on `/new-project`
- @mention notifications when agent needs input
- Rich embeds for agent status and progress
- Channel archival for inactive projects
