//! Memory auto-update hooks for agent actions.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::memory::file::{
    ActiveContextData, ArchitectureItem, DecisionItem, GotchaItem, MemoryError, ProjectMemory,
    RejectedItem,
};
use crate::memory::log::{ChangeType, DecisionEntry, DecisionLog};

const RATE_LIMIT_DURATION: Duration = Duration::from_secs(60);

/// Event that triggers a memory update.
#[derive(Debug, Clone)]
pub enum UpdateEvent {
    /// New file created in src/.
    FileCreated {
        /// Path to the created file.
        path: String,
        /// Component name (derived from path).
        component: Option<String>,
    },
    /// Significant decision made.
    DecisionMade {
        /// The decision.
        decision: String,
        /// Rationale for the decision.
        rationale: String,
    },
    /// Bug fix or workaround applied.
    BugfixApplied {
        /// Description of the gotcha.
        gotcha: String,
        /// Context or explanation.
        context: String,
    },
    /// Approach rejected.
    ApproachRejected {
        /// The rejected approach.
        approach: String,
        /// Why it was rejected.
        reason: String,
    },
    /// Task context updated.
    ContextUpdated {
        /// Current task.
        current_task: Option<String>,
        /// Blockers.
        blockers: Option<String>,
        /// Next steps.
        next_steps: Option<String>,
    },
    /// Architecture component added.
    ArchitectureAdded {
        /// Component name.
        component: String,
        /// Component description.
        description: String,
    },
}

/// Trigger type for memory updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTrigger {
    /// File creation in src/.
    FileCreation,
    /// Architectural change.
    ArchitecturalChange,
    /// Bug fix with workaround.
    BugfixWorkaround,
    /// Explicit trade-off reasoning.
    TradeoffDecision,
    /// Task completion.
    TaskCompletion,
}

impl MemoryTrigger {
    /// Check if a file path should trigger a memory update.
    pub fn should_trigger_for_file(path: &str) -> bool {
        path.starts_with("src/") || path.contains("/src/")
    }
}

/// Memory hooks for automatic updates.
pub struct MemoryHooks {
    project_root: PathBuf,
    last_update: Arc<Mutex<Option<Instant>>>,
    conversation_id: String,
    agent_name: String,
}

impl MemoryHooks {
    /// Create new memory hooks for a project.
    pub fn new(
        project_root: impl Into<PathBuf>,
        conversation_id: impl Into<String>,
        agent_name: impl Into<String>,
    ) -> Self {
        Self {
            project_root: project_root.into(),
            last_update: Arc::new(Mutex::new(None)),
            conversation_id: conversation_id.into(),
            agent_name: agent_name.into(),
        }
    }

    /// Process an update event (async, non-blocking).
    pub async fn process_event(
        &self,
        event: UpdateEvent,
        message_id: &str,
    ) -> Result<bool, MemoryError> {
        if !self.check_rate_limit().await {
            debug!("Memory update rate limited");
            return Ok(false);
        }

        let result = self.apply_event(&event, message_id).await;

        if result.is_ok() {
            let mut last = self.last_update.lock().await;
            *last = Some(Instant::now());
        }

        result.map(|_| true)
    }

    /// Spawn an async task to process the event (fire-and-forget).
    pub fn process_event_async(self: Arc<Self>, event: UpdateEvent, message_id: String) {
        tokio::spawn(async move {
            if let Err(e) = self.process_event(event, &message_id).await {
                warn!("Memory update failed: {e}");
            }
        });
    }

    async fn check_rate_limit(&self) -> bool {
        let last = self.last_update.lock().await;
        match *last {
            Some(instant) => instant.elapsed() >= RATE_LIMIT_DURATION,
            None => true,
        }
    }

    async fn apply_event(&self, event: &UpdateEvent, message_id: &str) -> Result<(), MemoryError> {
        let memory_path = ProjectMemory::default_path(&self.project_root);
        let mut memory = if memory_path.exists() {
            ProjectMemory::load(&memory_path)?
        } else {
            let project_name = self
                .project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            ProjectMemory::new(project_name)
        };

        let log = DecisionLog::new(DecisionLog::default_path(&self.project_root));

        match event {
            UpdateEvent::FileCreated { path, component } => {
                let entry = DecisionEntry::new(
                    &self.conversation_id,
                    message_id,
                    ChangeType::Create,
                    format!("Created file: {path}"),
                    "New file added to project",
                    &self.agent_name,
                )
                .with_file(path);
                log.append(&entry)?;

                if let Some(comp) = component {
                    memory.architecture.push(ArchitectureItem {
                        component: comp.clone(),
                        description: format!("Component at {path}"),
                    });
                }
            }
            UpdateEvent::DecisionMade {
                decision,
                rationale,
            } => {
                let entry = DecisionEntry::new(
                    &self.conversation_id,
                    message_id,
                    ChangeType::Decide,
                    decision,
                    rationale,
                    &self.agent_name,
                );
                log.append(&entry)?;

                let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                memory.key_decisions.push(DecisionItem {
                    date,
                    decision: decision.clone(),
                    rationale: rationale.clone(),
                });
            }
            UpdateEvent::BugfixApplied { gotcha, context } => {
                let entry = DecisionEntry::new(
                    &self.conversation_id,
                    message_id,
                    ChangeType::Bugfix,
                    gotcha,
                    context,
                    &self.agent_name,
                );
                log.append(&entry)?;

                memory.gotchas.push(GotchaItem {
                    gotcha: gotcha.clone(),
                    context: context.clone(),
                });
            }
            UpdateEvent::ApproachRejected { approach, reason } => {
                let entry = DecisionEntry::new(
                    &self.conversation_id,
                    message_id,
                    ChangeType::Decide,
                    format!("Rejected: {approach}"),
                    reason,
                    &self.agent_name,
                );
                log.append(&entry)?;

                memory.rejected_approaches.push(RejectedItem {
                    approach: approach.clone(),
                    reason: reason.clone(),
                });
            }
            UpdateEvent::ContextUpdated {
                current_task,
                blockers,
                next_steps,
            } => {
                memory.active_context = ActiveContextData {
                    current_task: current_task.clone(),
                    blockers: blockers.clone(),
                    next_steps: next_steps.clone(),
                };
            }
            UpdateEvent::ArchitectureAdded {
                component,
                description,
            } => {
                let entry = DecisionEntry::new(
                    &self.conversation_id,
                    message_id,
                    ChangeType::Architecture,
                    format!("Added component: {component}"),
                    description,
                    &self.agent_name,
                );
                log.append(&entry)?;

                memory.architecture.push(ArchitectureItem {
                    component: component.clone(),
                    description: description.clone(),
                });
            }
        }

        memory.save(&memory_path)?;
        Ok(())
    }

    /// Get the project root path.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_hooks(temp_dir: &TempDir) -> MemoryHooks {
        MemoryHooks::new(temp_dir.path(), "test-conv", "test-agent")
    }

    #[tokio::test]
    async fn test_process_file_created_event() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::FileCreated {
            path: "src/memory/mod.rs".to_string(),
            component: Some("memory".to_string()),
        };

        let result = hooks.process_event(event, "msg-1").await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed"));

        let memory_path = ProjectMemory::default_path(temp_dir.path());
        assert!(memory_path.exists());

        let memory = ProjectMemory::load(&memory_path).expect("load memory");
        assert_eq!(memory.architecture.len(), 1);
        assert_eq!(memory.architecture[0].component, "memory");
    }

    #[tokio::test]
    async fn test_process_decision_made_event() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::DecisionMade {
            decision: "Use markdown for memory files".to_string(),
            rationale: "Human-readable and easy to parse".to_string(),
        };

        hooks.process_event(event, "msg-1").await.expect("process");

        let memory =
            ProjectMemory::load(ProjectMemory::default_path(temp_dir.path())).expect("load memory");
        assert_eq!(memory.key_decisions.len(), 1);
        assert!(memory.key_decisions[0].decision.contains("markdown"));
    }

    #[tokio::test]
    async fn test_process_bugfix_event() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::BugfixApplied {
            gotcha: "UTF-8 BOM".to_string(),
            context: "Some editors add BOM, strip it on load".to_string(),
        };

        hooks.process_event(event, "msg-1").await.expect("process");

        let memory =
            ProjectMemory::load(ProjectMemory::default_path(temp_dir.path())).expect("load memory");
        assert_eq!(memory.gotchas.len(), 1);
        assert_eq!(memory.gotchas[0].gotcha, "UTF-8 BOM");
    }

    #[tokio::test]
    async fn test_process_rejected_approach_event() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::ApproachRejected {
            approach: "JSON format".to_string(),
            reason: "Not human-readable enough".to_string(),
        };

        hooks.process_event(event, "msg-1").await.expect("process");

        let memory =
            ProjectMemory::load(ProjectMemory::default_path(temp_dir.path())).expect("load memory");
        assert_eq!(memory.rejected_approaches.len(), 1);
        assert_eq!(memory.rejected_approaches[0].approach, "JSON format");
    }

    #[tokio::test]
    async fn test_process_context_updated_event() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::ContextUpdated {
            current_task: Some("Implementing memory system".to_string()),
            blockers: Some("None".to_string()),
            next_steps: Some("Add tests".to_string()),
        };

        hooks.process_event(event, "msg-1").await.expect("process");

        let memory =
            ProjectMemory::load(ProjectMemory::default_path(temp_dir.path())).expect("load memory");
        assert_eq!(
            memory.active_context.current_task,
            Some("Implementing memory system".to_string())
        );
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event1 = UpdateEvent::DecisionMade {
            decision: "First decision".to_string(),
            rationale: "First rationale".to_string(),
        };
        let event2 = UpdateEvent::DecisionMade {
            decision: "Second decision".to_string(),
            rationale: "Second rationale".to_string(),
        };

        let result1 = hooks.process_event(event1, "msg-1").await.expect("process");
        assert!(result1);

        let result2 = hooks.process_event(event2, "msg-2").await.expect("process");
        assert!(!result2);

        let memory =
            ProjectMemory::load(ProjectMemory::default_path(temp_dir.path())).expect("load memory");
        assert_eq!(memory.key_decisions.len(), 1);
    }

    #[tokio::test]
    async fn test_decision_log_created() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks = test_hooks(&temp_dir);

        let event = UpdateEvent::FileCreated {
            path: "src/lib.rs".to_string(),
            component: None,
        };

        hooks.process_event(event, "msg-1").await.expect("process");

        let log_path = DecisionLog::default_path(temp_dir.path());
        assert!(log_path.exists());

        let log = DecisionLog::new(&log_path);
        let entries = log.read_all().expect("read log");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_path, Some("src/lib.rs".to_string()));
    }

    #[test]
    fn test_should_trigger_for_file() {
        assert!(MemoryTrigger::should_trigger_for_file("src/main.rs"));
        assert!(MemoryTrigger::should_trigger_for_file(
            "crates/foo/src/lib.rs"
        ));
        assert!(!MemoryTrigger::should_trigger_for_file("README.md"));
        assert!(!MemoryTrigger::should_trigger_for_file("Cargo.toml"));
    }
}
