# Cuttlefish Architecture Overview

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Client Interfaces                               │
├─────────────────┬─────────────────┬─────────────────┬───────────────────────┤
│     WebUI       │      TUI        │    Discord      │    Future Clients     │
│  (Nuxt 3/Vue)   │   (ratatui)     │   (serenity)    │   (Mobile, IDE, etc)  │
└────────┬────────┴────────┬────────┴────────┬────────┴───────────┬───────────┘
         │                 │                 │                     │
         └─────────────────┴─────────────────┴─────────────────────┘
                                    │
                          WebSocket / REST API
                                    │
┌───────────────────────────────────▼─────────────────────────────────────────┐
│                            cuttlefish-api                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  WebSocket  │  │  REST API   │  │   Safety    │  │  Approval Registry  │ │
│  │   Handler   │  │   Routes    │  │   Routes    │  │   (Pending Actions) │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────┼────────────────────┼────────────┘
          │                │                │                    │
          └────────────────┴────────────────┴────────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────────┐
│                          cuttlefish-agents                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                        Workflow Engine                                   ││
│  │  ┌────────────┐    ┌────────┐    ┌────────┐                             ││
│  │  │Orchestrator│───▶│ Coder  │◀──▶│ Critic │                             ││
│  │  └────────────┘    └────────┘    └────────┘                             ││
│  │         │                                                                ││
│  │  ┌──────▼──────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐              ││
│  │  │   Planner   │  │ Explorer │  │Librarian │  │ DevOps  │              ││
│  │  └─────────────┘  └──────────┘  └──────────┘  └─────────┘              ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Message Bus │  │   Safety    │  │   Memory    │  │  Prompt Registry    │ │
│  │  (Tokio)    │  │   Gates     │  │   System    │  │   (YAML/Markdown)   │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
          │                │                │                    │
          └────────────────┴────────────────┴────────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────────┐
│                         Provider & Infrastructure                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │    Providers    │  │     Sandbox     │  │    Database     │              │
│  │  (11 backends)  │  │    (Docker)     │  │    (SQLite)     │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Request Lifecycle

### Chat Message Flow

1. **Client** sends WebSocket `chat` message with `project_id` and `content`
2. **WebSocket Handler** (`ws.rs`) receives and parses the message
3. **execute_workflow()** is called with the input
4. **WorkflowEngine** executes the agent pipeline:
   - Orchestrator analyzes the task
   - (Optional) Planner creates implementation plan
   - Coder implements the task
   - Critic reviews the output
   - Loop until approved or max iterations
5. **Log entries** are streamed back via WebSocket during execution
6. **Final response** is sent as a `response` message

### Safety Approval Flow

1. **Action** is proposed (file write, command execution, etc.)
2. **ActionGate** evaluates confidence score against thresholds
3. **Decision**:
   - `AutoApprove` (confidence >= 0.9): Action proceeds immediately
   - `PromptUser` (confidence 0.5-0.9): Pending approval created
   - `Block` (confidence < 0.5): Action rejected
4. For `PromptUser`:
   - `PendingApproval` registered in ApprovalRegistry
   - `pending_approval` message sent to client
   - Workflow waits for user decision
   - User sends `approve` or `reject` via WebSocket
   - ApprovalRegistry resolves the pending action
   - Workflow continues or aborts

## Crate Dependency Graph

```
cuttlefish-core (no internal deps)
       │
       ├──► cuttlefish-db
       ├──► cuttlefish-providers
       ├──► cuttlefish-sandbox
       ├──► cuttlefish-vcs
       └──► cuttlefish-tunnel
              │
              └──► cuttlefish-agents
                         │
                         ├──► cuttlefish-api
                         ├──► cuttlefish-discord
                         └──► cuttlefish-tui
                                   │
                                   └──► cuttlefish-rs (binary)
```

## Data Flow

### Configuration
- `cuttlefish.toml` → `CuttlefishConfig` → `AppState`
- Providers initialized from config at startup
- Agent prompts loaded from `./prompts/` directory

### State Management
- `AppState` shared across all HTTP/WebSocket handlers
- `ProjectSession` tracks active projects and connected clients
- `ApprovalRegistry` tracks pending safety approvals
- `DashMap` used for concurrent session access

### Persistence
- SQLite database for projects, conversations, usage tracking
- File-based memory system for project context
- Checkpoints for state snapshots and rollback
