# V1 Agent Memory System — Work Breakdown Structure

## Overview
- **Plan**: `.sisyphus/plans/v1-memory.md`
- **Total Tasks**: 13 + 4 verification = 17 task groups
- **Estimated Duration**: 4-5 days
- **Parallel Execution**: YES - 4 internal waves

## Quick Reference

| Wave | Tasks | Category | Parallel |
|------|-------|----------|----------|
| 1 | T1: Memory file format, T2: Decision log, T3: Auto-update hooks | deep, unspecified-high | YES |
| 2 | T4: Decision indexing, T5: Why command, T6: Conversation context | deep, unspecified-high | YES |
| 3 | T7: Branch architecture, T8: Fork, T9: Restore, T10: Management | deep, quick | YES |
| 4 | T11: Memory API, T12: CLI commands, T13: Agent prompts | unspecified-high, quick | YES |
| FINAL | F1-F4: Verification | oracle, deep | YES |

## Task Summary

### Wave 1: Memory Files
- **T1**: Memory file format (`.cuttlefish/memory.md`) with sections: Summary, Decisions, Architecture, Gotchas, Rejected
- **T2**: Decision log structure (`.cuttlefish/decisions.jsonl`) with JSONL append-only format
- **T3**: Auto-update hooks triggered on file creation, decisions, errors

### Wave 2: Why Command
- **T4**: In-memory indexes for file_path, conversation_id, timestamp lookups
- **T5**: `why <file>` returns decisions + conversation excerpts
- **T6**: Retrieve conversation context around decision points

### Wave 3: State Branching
- **T7**: `StateBranch` struct with git_ref, container_snapshot, memory_snapshot
- **T8**: `create_branch()` forks git + container + memory
- **T9**: `restore_branch()` returns to forked state
- **T10**: List, delete, compare branches

### Wave 4: Integration
- **T11**: 7 memory/branch API endpoints
- **T12**: `cuttlefish why`, `cuttlefish memory`, `cuttlefish branch` commands
- **T13**: Update agent prompts to read/write memory

## Dependencies
```
T1 → T2 → T4 → T5 → T11
T1 → T3 → T4
T7 → T8 → T9 → T10 → T11
T3 → T13
```

## Files to Create/Modify
- `crates/cuttlefish-agents/src/memory/mod.rs`
- `crates/cuttlefish-agents/src/memory/file.rs`
- `crates/cuttlefish-agents/src/memory/log.rs`
- `crates/cuttlefish-agents/src/memory/hooks.rs`
- `crates/cuttlefish-agents/src/memory/index.rs`
- `crates/cuttlefish-agents/src/memory/why.rs`
- `crates/cuttlefish-agents/src/memory/branch.rs`
- `crates/cuttlefish-api/src/memory_routes.rs`
- `prompts/*.md` (updates)

## Memory File Format
```markdown
# Project Memory: {project_name}

## Summary
> One-paragraph project description

## Key Decisions
- **{date}**: {decision} — {rationale}

## Architecture
- {component}: {description}

## Gotchas & Lessons
- {gotcha}: {context}

## Rejected Approaches
- {approach}: {why rejected}

## Active Context
- Currently working on: {task}
```

## Success Criteria
- Memory file created and updated by agents
- `why` command traces decisions to conversations
- State branching forks container + git + memory
- Branch restore works completely
- No sensitive data in memory files
