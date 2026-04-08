//! Context compaction to prevent context window exhaustion.
//!
//! Implements strategies to reduce context size while preserving important information:
//! 1. Truncate large tool results (keep summary)
//! 2. Summarize old conversation segments
//! 3. Remove redundant assistant messages

use cuttlefish_core::traits::provider::{Message, MessageRole};
use tracing::{debug, info};

use super::counter::TokenCounter;

/// Configuration for context compaction.
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Token threshold to start pruning (soft limit).
    pub prune_minimum: usize,
    /// Token threshold for urgent pruning (hard limit).
    pub prune_protect: usize,
    /// Maximum context size (absolute limit).
    pub max_context: usize,
    /// Maximum characters to keep from tool results before truncation.
    pub tool_result_max_chars: usize,
    /// Number of recent message pairs to protect from removal.
    pub protected_recent_pairs: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            prune_minimum: 20_000,       // Start pruning at 20k tokens
            prune_protect: 40_000,       // Urgent pruning at 40k tokens
            max_context: 100_000,        // Hard limit at 100k tokens
            tool_result_max_chars: 2000, // Truncate tool results over 2k chars
            protected_recent_pairs: 5,   // Protect last 5 user/assistant pairs
        }
    }
}

impl CompactionConfig {
    /// Create a conservative config for smaller context windows.
    pub fn conservative() -> Self {
        Self {
            prune_minimum: 10_000,
            prune_protect: 20_000,
            max_context: 50_000,
            tool_result_max_chars: 1000,
            protected_recent_pairs: 3,
        }
    }

    /// Create an aggressive config for very long conversations.
    pub fn aggressive() -> Self {
        Self {
            prune_minimum: 30_000,
            prune_protect: 60_000,
            max_context: 150_000,
            tool_result_max_chars: 500,
            protected_recent_pairs: 3,
        }
    }
}

/// Result of a compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Tokens before compaction.
    pub tokens_before: usize,
    /// Tokens after compaction.
    pub tokens_after: usize,
    /// Number of messages removed.
    pub messages_removed: usize,
    /// Number of tool results truncated.
    pub tool_results_truncated: usize,
    /// Whether summarization was performed.
    pub summarized: bool,
}

/// Context compactor that manages conversation history size.
pub struct ContextCompactor {
    config: CompactionConfig,
    counter: TokenCounter,
}

impl ContextCompactor {
    /// Create a new compactor with default configuration.
    pub fn new() -> Self {
        Self {
            config: CompactionConfig::default(),
            counter: TokenCounter::new(),
        }
    }

    /// Create a compactor with custom configuration.
    pub fn with_config(config: CompactionConfig) -> Self {
        Self {
            config,
            counter: TokenCounter::new(),
        }
    }

    /// Check if compaction is needed based on current token count.
    pub fn needs_compaction(&self, messages: &[Message]) -> bool {
        let current_tokens = self.counter.count_messages(messages);
        current_tokens >= self.config.prune_minimum
    }

    /// Check if urgent compaction is needed.
    pub fn needs_urgent_compaction(&self, messages: &[Message]) -> bool {
        let current_tokens = self.counter.count_messages(messages);
        current_tokens >= self.config.prune_protect
    }

    /// Compact the message history.
    ///
    /// Applies strategies in order:
    /// 1. Truncate large tool results
    /// 2. Remove old assistant messages (keep user messages for context)
    /// 3. If still over limit, remove older conversation turns
    pub fn compact(&self, messages: &mut Vec<Message>) -> CompactionResult {
        let tokens_before = self.counter.count_messages(messages);

        if tokens_before < self.config.prune_minimum {
            return CompactionResult {
                tokens_before,
                tokens_after: tokens_before,
                messages_removed: 0,
                tool_results_truncated: 0,
                summarized: false,
            };
        }

        info!(
            tokens = tokens_before,
            threshold = self.config.prune_minimum,
            "Starting context compaction"
        );

        let mut tool_results_truncated = 0;
        let messages_before = messages.len();

        // Strategy 1: Truncate large tool results
        tool_results_truncated += self.truncate_tool_results(messages);

        let tokens_after_truncation = self.counter.count_messages(messages);
        debug!(
            tokens_before = tokens_before,
            tokens_after = tokens_after_truncation,
            truncated = tool_results_truncated,
            "After tool result truncation"
        );

        // Strategy 2: Remove old messages if still over threshold
        if tokens_after_truncation >= self.config.prune_protect {
            self.remove_old_messages(messages);
        }

        let tokens_after = self.counter.count_messages(messages);
        let messages_removed = messages_before.saturating_sub(messages.len());

        info!(
            tokens_before = tokens_before,
            tokens_after = tokens_after,
            messages_removed = messages_removed,
            "Context compaction complete"
        );

        CompactionResult {
            tokens_before,
            tokens_after,
            messages_removed,
            tool_results_truncated,
            summarized: false, // TODO: implement summarization
        }
    }

    /// Truncate large tool results, keeping a summary.
    fn truncate_tool_results(&self, messages: &mut [Message]) -> usize {
        let mut truncated = 0;
        let max_chars = self.config.tool_result_max_chars;

        for msg in messages.iter_mut() {
            // Tool results are typically in assistant messages with specific patterns
            // or could be identified by role if we had a ToolResult role
            if msg.content.len() > max_chars {
                // Check if this looks like a tool result (heuristic)
                if self.looks_like_tool_result(&msg.content) {
                    let original_len = msg.content.len();
                    let preview = &msg.content[..max_chars.min(msg.content.len())];

                    // Find a good break point (newline or sentence end)
                    let break_point = preview
                        .rfind('\n')
                        .or_else(|| preview.rfind(". "))
                        .unwrap_or(max_chars.min(preview.len()));

                    let truncated_content = &msg.content[..break_point];
                    msg.content = format!(
                        "{}\n\n... [truncated {} chars, showing first {}]",
                        truncated_content,
                        original_len - break_point,
                        break_point
                    );
                    truncated += 1;
                }
            }
        }

        truncated
    }

    /// Heuristic to detect tool results that can be truncated.
    fn looks_like_tool_result(&self, content: &str) -> bool {
        // Tool results often have these patterns
        content.starts_with("Found ")
            || content.starts_with("Error:")
            || content.starts_with("stdout:")
            || content.starts_with("Successfully ")
            || content.contains("\n---\n") // grep results
            || content.lines().count() > 50 // Many lines suggests file/command output
    }

    /// Remove old messages while protecting recent conversation.
    fn remove_old_messages(&self, messages: &mut Vec<Message>) {
        let protected_count = self.config.protected_recent_pairs * 2; // User + Assistant pairs

        if messages.len() <= protected_count {
            return; // Nothing to remove
        }

        // Find the cutoff point - protect the last N message pairs
        let cutoff = messages.len().saturating_sub(protected_count);

        // Remove old assistant messages first (user messages provide context)
        let mut indices_to_remove: Vec<usize> = Vec::new();

        for (i, msg) in messages.iter().enumerate().take(cutoff) {
            if msg.role == MessageRole::Assistant {
                indices_to_remove.push(i);
            }
        }

        // Remove from end to start to preserve indices
        for i in indices_to_remove.into_iter().rev() {
            messages.remove(i);
        }

        // If still over limit, remove old user messages too
        let current_tokens = self.counter.count_messages(messages);
        if current_tokens >= self.config.max_context {
            let protected_count = self.config.protected_recent_pairs * 2;
            if messages.len() > protected_count {
                let to_remove = messages.len() - protected_count;
                messages.drain(0..to_remove);
            }
        }
    }

    /// Get current token count.
    pub fn current_tokens(&self, messages: &[Message]) -> usize {
        self.counter.count_messages(messages)
    }

    /// Get the token counter.
    pub fn counter(&self) -> &TokenCounter {
        &self.counter
    }

    /// Get the configuration.
    pub fn config(&self) -> &CompactionConfig {
        &self.config
    }
}

impl Default for ContextCompactor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
        }
    }

    #[test]
    fn test_no_compaction_needed() {
        let compactor = ContextCompactor::new();
        let mut messages = vec![
            make_message(MessageRole::User, "Hello"),
            make_message(MessageRole::Assistant, "Hi there!"),
        ];

        let result = compactor.compact(&mut messages);
        assert_eq!(result.messages_removed, 0);
        assert_eq!(result.tool_results_truncated, 0);
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_needs_compaction() {
        let config = CompactionConfig {
            prune_minimum: 100,
            ..Default::default()
        };
        let compactor = ContextCompactor::with_config(config);

        // Small messages - no compaction needed
        let small_messages = vec![make_message(MessageRole::User, "Hello")];
        assert!(!compactor.needs_compaction(&small_messages));

        // Large messages - compaction needed
        let large_messages = vec![make_message(MessageRole::User, &"a".repeat(1000))];
        assert!(compactor.needs_compaction(&large_messages));
    }

    #[test]
    fn test_truncate_tool_results() {
        let config = CompactionConfig {
            prune_minimum: 10, // Force compaction
            tool_result_max_chars: 100,
            ..Default::default()
        };
        let compactor = ContextCompactor::with_config(config);

        let long_output = format!("Found 50 files:\n{}", "file.rs\n".repeat(100));
        let mut messages = vec![make_message(MessageRole::Assistant, &long_output)];

        let result = compactor.compact(&mut messages);
        assert_eq!(result.tool_results_truncated, 1);
        assert!(messages[0].content.contains("[truncated"));
    }

    #[test]
    fn test_remove_old_messages() {
        let config = CompactionConfig {
            prune_minimum: 10,
            prune_protect: 20,
            protected_recent_pairs: 2,
            ..Default::default()
        };
        let compactor = ContextCompactor::with_config(config);

        // Create many messages
        let mut messages: Vec<Message> = (0..20)
            .map(|i| {
                if i % 2 == 0 {
                    make_message(MessageRole::User, &format!("User message {}", i))
                } else {
                    make_message(MessageRole::Assistant, &format!("Assistant message {}", i))
                }
            })
            .collect();

        let original_len = messages.len();
        compactor.compact(&mut messages);

        // Should have removed some messages but kept protected pairs
        assert!(messages.len() < original_len);
        assert!(messages.len() >= 4); // At least 2 pairs protected
    }

    #[test]
    fn test_looks_like_tool_result() {
        let compactor = ContextCompactor::new();

        assert!(compactor.looks_like_tool_result("Found 10 files:\nfile1.rs"));
        assert!(compactor.looks_like_tool_result("Error: file not found"));
        assert!(compactor.looks_like_tool_result("stdout: hello\nstderr: "));
        assert!(compactor.looks_like_tool_result("Successfully wrote 100 bytes"));

        // Many lines suggests output
        let many_lines = "line\n".repeat(60);
        assert!(compactor.looks_like_tool_result(&many_lines));

        // Normal conversation shouldn't match
        assert!(!compactor.looks_like_tool_result("I think we should refactor this"));
    }

    #[test]
    fn test_compaction_result() {
        let config = CompactionConfig {
            prune_minimum: 10,
            tool_result_max_chars: 50,
            ..Default::default()
        };
        let compactor = ContextCompactor::with_config(config);

        let mut messages = vec![make_message(
            MessageRole::Assistant,
            &format!("Found 100 files:\n{}", "x".repeat(500)),
        )];

        let result = compactor.compact(&mut messages);
        assert!(result.tokens_after <= result.tokens_before);
    }

    #[test]
    fn test_config_variants() {
        let default = CompactionConfig::default();
        let conservative = CompactionConfig::conservative();
        let aggressive = CompactionConfig::aggressive();

        assert!(conservative.prune_minimum < default.prune_minimum);
        assert!(aggressive.prune_minimum > default.prune_minimum);
        assert!(conservative.tool_result_max_chars < default.tool_result_max_chars);
    }
}
