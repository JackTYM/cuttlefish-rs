# WebSocket Protocol Specification

## Connection

**Endpoint**: `ws://<host>:<port>/ws`

**Authentication**: API key passed as query parameter `?key=<api_key>`

## Message Format

All messages are JSON with a `type` field for routing.

## Client → Server Messages

### Chat Message
Send a message to an agent for a specific project.

```json
{
  "type": "chat",
  "project_id": "uuid-string",
  "content": "Create a hello world function"
}
```

### Ping
Keep-alive message.

```json
{
  "type": "ping"
}
```

### Approve
Approve a pending action.

```json
{
  "type": "approve",
  "action_id": "uuid-string"
}
```

### Reject
Reject a pending action.

```json
{
  "type": "reject",
  "action_id": "uuid-string",
  "reason": "Optional rejection reason"
}
```

### Subscribe
Subscribe to updates for a project.

```json
{
  "type": "subscribe",
  "project_id": "uuid-string"
}
```

### Unsubscribe
Unsubscribe from project updates.

```json
{
  "type": "unsubscribe",
  "project_id": "uuid-string"
}
```

## Server → Client Messages

### Response
Agent response to a chat message.

```json
{
  "type": "response",
  "project_id": "uuid-string",
  "agent": "orchestrator|coder|critic|planner|explorer|librarian|devops|workflow",
  "content": "Agent output content"
}
```

### Build Log
Streaming build/execution log line.

```json
{
  "type": "build_log",
  "project_id": "uuid-string",
  "line": "Log line content"
}
```

### Diff
File diff preview for pending changes.

```json
{
  "type": "diff",
  "project_id": "uuid-string",
  "patch": "Unified diff format patch"
}
```

### Pending Approval
An action requires user approval before proceeding.

```json
{
  "type": "pending_approval",
  "action_id": "uuid-string",
  "project_id": "uuid-string",
  "action_type": "FileWrite|FileDelete|BashCommand|GitOperation|ConfigChange",
  "description": "Human-readable description of the action",
  "path": "src/main.rs",
  "command": null,
  "confidence": 0.65,
  "confidence_reasoning": "Modifies existing source file",
  "risk_factors": [
    {
      "type": "Risk Level",
      "description": "Risk Level: 35%"
    }
  ],
  "created_at": "2024-01-15T10:00:00Z",
  "timeout_secs": 300,
  "has_diff": true
}
```

**Fields**:
- `action_id`: Unique ID for this action (use to approve/reject)
- `action_type`: Category of action
- `confidence`: Score from 0.0 to 1.0
- `timeout_secs`: How long before the approval times out
- `has_diff`: Whether a diff preview is available

### Approval Resolved
A pending approval has been resolved (approved, rejected, or timed out).

```json
{
  "type": "approval_resolved",
  "action_id": "uuid-string"
}
```

### Log Entry
Real-time log entry from agent activity.

```json
{
  "type": "log_entry",
  "id": "uuid-string",
  "timestamp": "2024-01-15T10:00:00Z",
  "agent": "coder",
  "action": "Writing code for feature X",
  "level": "info|warn|error",
  "project": "project-name-or-id",
  "context": "Optional additional context",
  "stack_trace": "Optional stack trace for errors"
}
```

### Pong
Response to ping.

```json
{
  "type": "pong"
}
```

### Error
Error message.

```json
{
  "type": "error",
  "message": "Error description"
}
```

## Typical Message Sequences

### Chat Request/Response

```
Client                          Server
  │                               │
  │──── chat ────────────────────▶│
  │                               │
  │◀─── log_entry (orchestrator)──│
  │◀─── log_entry (coder) ────────│
  │◀─── log_entry (critic) ───────│
  │                               │
  │◀─── response ─────────────────│
  │                               │
```

### Action Requiring Approval

```
Client                          Server
  │                               │
  │──── chat ────────────────────▶│
  │                               │
  │◀─── log_entry ────────────────│
  │◀─── pending_approval ─────────│
  │                               │
  │──── approve ─────────────────▶│
  │                               │
  │◀─── approval_resolved ────────│
  │◀─── log_entry ────────────────│
  │◀─── response ─────────────────│
  │                               │
```

### Subscription

```
Client                          Server
  │                               │
  │──── subscribe ───────────────▶│
  │                               │
  │◀─── log_entry (from others)───│
  │◀─── pending_approval ─────────│
  │                               │
  │──── unsubscribe ─────────────▶│
  │                               │
```

## Error Handling

- Invalid JSON: Server sends `error` message with parse details
- Unknown message type: Server sends `error` message
- Action not found for approve/reject: Logged as warning, `approval_resolved` still sent
- Connection errors: Client should implement reconnection logic

## Keep-Alive

Clients should send `ping` messages periodically (recommended: every 30 seconds) to keep the connection alive. Server responds with `pong`.
