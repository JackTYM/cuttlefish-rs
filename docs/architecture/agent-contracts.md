# Agent Contracts

## Overview

Cuttlefish uses a multi-agent system where specialized agents collaborate to complete tasks. Each agent has a defined role, capabilities, and communication interface.

## Agent Roles

| Agent | Role | Default Category | Description |
|-------|------|------------------|-------------|
| Orchestrator | Coordinator | `deep` | Receives tasks, creates plans, dispatches to other agents |
| Planner | Architect | `ultrabrain` | Creates detailed implementation plans for complex tasks |
| Coder | Implementer | `deep` | Writes code, runs builds, executes tests |
| Critic | Reviewer | `unspecified-high` | Reviews code changes, approves or requests revisions |
| Explorer | Searcher | `quick` | Searches codebases for patterns and code |
| Librarian | Documentation | `quick` | Retrieves documentation for libraries and APIs |
| DevOps | Operations | `unspecified-high` | Handles builds, deployments, infrastructure |

## Model Categories

Categories map to model selection for optimal cost/quality balance.

| Category | Use Case | Typical Model |
|----------|----------|---------------|
| `ultrabrain` | Hard logic, architecture decisions | claude-opus-4-6 |
| `deep` | Complex autonomous work | claude-sonnet-4-6 |
| `quick` | Fast simple tasks | claude-haiku-4-5 |
| `visual` | Frontend, UI/UX design | gemini-2.0-flash |
| `unspecified-high` | General higher effort | claude-sonnet-4-6 |
| `unspecified-low` | General lower effort | claude-haiku-4-5 |

## Workflow Engine

The core workflow follows this pattern:

```
                    ┌─────────────────┐
                    │   Orchestrator  │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │     Planner     │ (optional)
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             │
       ┌─────────────┐                      │
       │    Coder    │◀─────────────────────┤
       └──────┬──────┘                      │
              │                             │
              ▼                             │
       ┌─────────────┐      reject          │
       │   Critic    │──────────────────────┘
       └──────┬──────┘
              │ approve
              ▼
         ┌─────────┐
         │  Done   │
         └─────────┘
```

**Loop limits**: Maximum 5 Coder↔Critic iterations per task (configurable).

## Agent Context

Each agent execution receives an `AgentContext`:

```rust
pub struct AgentContext {
    /// Unique invocation ID
    pub invocation_id: Uuid,
    /// Project ID this execution belongs to
    pub project_id: Uuid,
    /// Working directory for file operations
    pub working_dir: PathBuf,
    /// Tools available to this agent
    pub available_tools: Vec<String>,
    /// Conversation history
    pub messages: Vec<Message>,
}
```

## Agent Output

Each agent returns an `AgentOutput`:

```rust
pub struct AgentOutput {
    /// Main content output
    pub content: String,
    /// Files that were changed
    pub files_changed: Vec<PathBuf>,
    /// Commands that were run
    pub commands_run: Vec<String>,
    /// Whether execution succeeded
    pub success: bool,
    /// Additional metadata
    pub metadata: serde_json::Value,
}
```

## Message Bus

Agents communicate via `TokioMessageBus` using broadcast channels.

**Message Structure**:
```rust
pub struct BusMessage {
    pub from: String,      // Sender agent name
    pub to: String,        // Recipient agent name ("*" for broadcast)
    pub kind: String,      // Message type
    pub payload: Value,    // JSON payload
}
```

## Prompt Registry

Agent prompts are loaded from markdown files with YAML frontmatter:

```markdown
---
name: coder
description: Code implementation agent
tools:
  - read_file
  - write_file
  - execute_command
category: deep
---

You are the Coder agent. Your role is to implement code changes
based on the task description and any planning provided.

## Guidelines

1. Write clean, idiomatic code
2. Follow existing project conventions
3. Include appropriate error handling
4. Add comments only where necessary
```

**File location**: `./prompts/<agent_name>.md`

## Tool Definitions

Agents can use tools defined in the prompt or available in the context.

### Standard Tools

| Tool | Description | Arguments |
|------|-------------|-----------|
| `read_file` | Read file contents | `path` |
| `write_file` | Write/create file | `path`, `content` |
| `edit_file` | Apply edits to file | `path`, `edits` |
| `execute_command` | Run shell command | `command`, `args`, `working_dir` |
| `list_directory` | List directory contents | `path` |
| `search_files` | Search for files by pattern | `pattern`, `path` |

## Safety Integration

Before executing actions, the safety system evaluates confidence:

1. **ConfidenceCalculator** computes a score based on:
   - Action type (write, delete, command, etc.)
   - File path (high-impact vs low-impact)
   - Command risk (safe vs dangerous)
   - Historical precedent

2. **ActionGate** decides based on thresholds:
   - Score >= 0.9: Auto-approve
   - Score 0.5-0.9: Prompt user
   - Score < 0.5: Block

3. **ApprovalRegistry** handles user decisions:
   - Tracks pending approvals
   - Waits for user input
   - Resolves with approve/reject/timeout

## Error Handling

Agents propagate errors via `AgentError`:

```rust
pub struct AgentError(pub String);
```

Common error cases:
- Provider API failure
- Prompt not found
- Tool execution failure
- Max iterations exceeded
- Safety gate blocked action
