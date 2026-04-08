//! Token counting for context management.
//!
//! Provides approximate token counts for messages to help manage context window limits.

use cuttlefish_core::traits::provider::Message;

/// Token counter for estimating context usage.
///
/// Uses character-based estimation (chars/4) which is reasonably accurate
/// for most models. Can be extended to use tiktoken for exact counts.
#[derive(Debug, Clone, Default)]
pub struct TokenCounter {
    /// Overhead tokens per message (role marker, formatting).
    pub message_overhead: usize,
}

impl TokenCounter {
    /// Create a new token counter with default settings.
    pub fn new() -> Self {
        Self {
            message_overhead: 4, // Approximate overhead for role + formatting
        }
    }

    /// Create a counter with custom message overhead.
    pub fn with_overhead(overhead: usize) -> Self {
        Self {
            message_overhead: overhead,
        }
    }

    /// Count approximate tokens in a text string.
    ///
    /// Uses chars/4 heuristic which is reasonably accurate for English text.
    /// For more accurate counts, consider integrating tiktoken-rs.
    pub fn count_text(&self, text: &str) -> usize {
        // chars/4 is a reasonable approximation for most tokenizers
        // Add 1 to avoid returning 0 for very short strings
        (text.chars().count() / 4).max(1)
    }

    /// Count tokens in a single message including overhead.
    pub fn count_message(&self, message: &Message) -> usize {
        self.count_text(&message.content) + self.message_overhead
    }

    /// Count total tokens across all messages.
    pub fn count_messages(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| self.count_message(m)).sum()
    }

    /// Count tokens in a system prompt.
    pub fn count_system(&self, system: &str) -> usize {
        self.count_text(system) + self.message_overhead
    }

    /// Estimate total context usage including system prompt and messages.
    pub fn estimate_context(&self, system: Option<&str>, messages: &[Message]) -> usize {
        let system_tokens = system.map(|s| self.count_system(s)).unwrap_or(0);
        let message_tokens = self.count_messages(messages);
        system_tokens + message_tokens
    }
}

/// Categories of messages for compaction decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageCategory {
    /// System prompt - never remove.
    System,
    /// Recent user message - protected from removal.
    RecentUser,
    /// Recent assistant message - protected from removal.
    RecentAssistant,
    /// Tool call from assistant.
    ToolCall,
    /// Tool result - can be truncated.
    ToolResult,
    /// Old user message - can be summarized.
    OldUser,
    /// Old assistant message - can be removed.
    OldAssistant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_core::traits::provider::MessageRole;

    #[test]
    fn test_count_text_empty() {
        let counter = TokenCounter::new();
        assert_eq!(counter.count_text(""), 1); // Minimum of 1
    }

    #[test]
    fn test_count_text_short() {
        let counter = TokenCounter::new();
        // "hello" = 5 chars, 5/4 = 1
        assert_eq!(counter.count_text("hello"), 1);
    }

    #[test]
    fn test_count_text_longer() {
        let counter = TokenCounter::new();
        // 100 chars / 4 = 25 tokens
        let text = "a".repeat(100);
        assert_eq!(counter.count_text(&text), 25);
    }

    #[test]
    fn test_count_message_includes_overhead() {
        let counter = TokenCounter::new();
        let msg = Message {
            role: MessageRole::User,
            content: "a".repeat(100), // 25 tokens
        };
        // 25 content + 4 overhead = 29
        assert_eq!(counter.count_message(&msg), 29);
    }

    #[test]
    fn test_count_messages_multiple() {
        let counter = TokenCounter::new();
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "a".repeat(100), // 25 + 4 = 29
            },
            Message {
                role: MessageRole::Assistant,
                content: "b".repeat(200), // 50 + 4 = 54
            },
        ];
        assert_eq!(counter.count_messages(&messages), 83);
    }

    #[test]
    fn test_estimate_context_with_system() {
        let counter = TokenCounter::new();
        let system = "a".repeat(400); // 100 + 4 = 104
        let messages = vec![Message {
            role: MessageRole::User,
            content: "b".repeat(100), // 25 + 4 = 29
        }];
        assert_eq!(counter.estimate_context(Some(&system), &messages), 133);
    }

    #[test]
    fn test_estimate_context_without_system() {
        let counter = TokenCounter::new();
        let messages = vec![Message {
            role: MessageRole::User,
            content: "a".repeat(100),
        }];
        assert_eq!(counter.estimate_context(None, &messages), 29);
    }
}
