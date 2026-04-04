//! Why command implementation for decision tracing.

use std::fmt::Write as FmtWrite;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::memory::index::DecisionIndex;
use crate::memory::log::DecisionEntry;

/// Target for the why command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyTarget {
    /// Query about a file.
    File(String),
    /// Query about a specific decision by ID.
    Decision(String),
    /// Query about a specific line in a file.
    Line {
        /// File path.
        file: String,
        /// Line number.
        line: usize,
    },
}

impl WhyTarget {
    /// Parse a target string (e.g., "src/lib.rs" or "src/lib.rs:42").
    pub fn parse(target: &str) -> Self {
        if let Some((file, line_str)) = target.rsplit_once(':')
            && let Ok(line) = line_str.parse::<usize>()
        {
            return Self::Line {
                file: file.to_string(),
                line,
            };
        }
        Self::File(target.to_string())
    }
}

/// A message in a conversation excerpt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcerptMessage {
    /// Message role (user, assistant, system).
    pub role: String,
    /// Message content (redacted if sensitive).
    pub content: String,
    /// Timestamp of the message.
    pub timestamp: Option<DateTime<Utc>>,
}

/// An excerpt from a conversation around a decision point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationExcerpt {
    /// Conversation ID.
    pub conversation_id: String,
    /// Message ID that triggered the decision.
    pub decision_message_id: String,
    /// Messages before the decision.
    pub messages_before: Vec<ExcerptMessage>,
    /// The decision message itself.
    pub decision_message: Option<ExcerptMessage>,
    /// Messages after the decision.
    pub messages_after: Vec<ExcerptMessage>,
}

impl ConversationExcerpt {
    /// Create a new empty excerpt.
    pub fn new(conversation_id: impl Into<String>, decision_message_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            decision_message_id: decision_message_id.into(),
            messages_before: Vec::new(),
            decision_message: None,
            messages_after: Vec::new(),
        }
    }

    /// Format the excerpt as markdown.
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        writeln!(output, "**Conversation:** {}", self.conversation_id)
            .expect("string write cannot fail");
        writeln!(output).expect("string write cannot fail");

        for msg in &self.messages_before {
            writeln!(output, "**{}:** {}", msg.role, msg.content)
                .expect("string write cannot fail");
        }

        if let Some(ref msg) = self.decision_message {
            writeln!(output, "**{} (decision):** {}", msg.role, msg.content)
                .expect("string write cannot fail");
        }

        for msg in &self.messages_after {
            writeln!(output, "**{}:** {}", msg.role, msg.content)
                .expect("string write cannot fail");
        }

        output
    }
}

/// Explanation returned by the why command.
#[derive(Debug, Clone)]
pub struct WhyExplanation {
    /// The target that was queried.
    pub target: String,
    /// Decisions related to the target.
    pub decisions: Vec<DecisionEntry>,
    /// Conversation excerpts for context.
    pub conversation_excerpts: Vec<ConversationExcerpt>,
    /// Human-readable summary.
    pub summary: String,
}

impl WhyExplanation {
    /// Format the explanation as markdown.
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        writeln!(output, "# Why: {}", self.target).expect("string write cannot fail");
        writeln!(output).expect("string write cannot fail");

        writeln!(output, "## Summary").expect("string write cannot fail");
        writeln!(output, "{}", self.summary).expect("string write cannot fail");
        writeln!(output).expect("string write cannot fail");

        if !self.decisions.is_empty() {
            writeln!(output, "## Decisions ({} total)", self.decisions.len())
                .expect("string write cannot fail");
            writeln!(output).expect("string write cannot fail");

            for (i, decision) in self.decisions.iter().enumerate() {
                writeln!(
                    output,
                    "### {}. {} ({})",
                    i + 1,
                    decision.summary,
                    decision.timestamp.format("%Y-%m-%d %H:%M")
                )
                .expect("string write cannot fail");
                writeln!(output, "- **Agent:** {}", decision.agent)
                    .expect("string write cannot fail");
                writeln!(output, "- **Type:** {:?}", decision.change_type)
                    .expect("string write cannot fail");
                if let Some(ref path) = decision.file_path {
                    writeln!(output, "- **File:** {path}").expect("string write cannot fail");
                }
                writeln!(output, "- **Reasoning:** {}", decision.reasoning)
                    .expect("string write cannot fail");
                writeln!(output).expect("string write cannot fail");
            }
        }

        if !self.conversation_excerpts.is_empty() {
            writeln!(output, "## Conversation Context").expect("string write cannot fail");
            writeln!(output).expect("string write cannot fail");

            for excerpt in &self.conversation_excerpts {
                writeln!(output, "{}", excerpt.to_markdown()).expect("string write cannot fail");
                writeln!(output, "---").expect("string write cannot fail");
            }
        }

        output
    }
}

/// Redact sensitive content from a string.
pub fn redact_sensitive(content: &str) -> String {
    let patterns = [
        (
            r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['"]?[\w-]+['"]?"#,
            "[REDACTED_API_KEY]",
        ),
        (
            r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]?[^\s'"]+['"]?"#,
            "[REDACTED_PASSWORD]",
        ),
        (
            r#"(?i)(secret|token)\s*[:=]\s*['"]?[\w-]+['"]?"#,
            "[REDACTED_SECRET]",
        ),
        (r"(?i)bearer\s+[\w.-]+", "Bearer [REDACTED]"),
        (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
        (r"ghp_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_TOKEN]"),
        (r"gho_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_TOKEN]"),
        (r"AKIA[0-9A-Z]{16}", "[REDACTED_AWS_KEY]"),
    ];

    let mut result = content.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }
    result
}

/// Execute the why command for a target.
pub fn why(index: &DecisionIndex, target: WhyTarget) -> WhyExplanation {
    let (target_str, decisions) = match &target {
        WhyTarget::File(path) => {
            let decisions: Vec<DecisionEntry> =
                index.find_by_file(path).into_iter().cloned().collect();
            (path.clone(), decisions)
        }
        WhyTarget::Decision(id) => {
            let decisions: Vec<DecisionEntry> = index
                .all()
                .iter()
                .filter(|e| e.id.to_string() == *id)
                .cloned()
                .collect();
            (format!("decision:{id}"), decisions)
        }
        WhyTarget::Line { file, line } => {
            let decisions: Vec<DecisionEntry> =
                index.find_by_file(file).into_iter().cloned().collect();
            (format!("{file}:{line}"), decisions)
        }
    };

    let summary = generate_summary(&target, &decisions);

    WhyExplanation {
        target: target_str,
        decisions,
        conversation_excerpts: Vec::new(),
        summary,
    }
}

fn generate_summary(target: &WhyTarget, decisions: &[DecisionEntry]) -> String {
    if decisions.is_empty() {
        return match target {
            WhyTarget::File(path) => format!("No recorded decisions found for file `{path}`."),
            WhyTarget::Decision(id) => format!("No decision found with ID `{id}`."),
            WhyTarget::Line { file, line } => {
                format!("No recorded decisions found for `{file}:{line}`.")
            }
        };
    }

    let mut summary = String::new();

    match target {
        WhyTarget::File(path) => {
            writeln!(
                summary,
                "Found {} decision(s) affecting `{}`.",
                decisions.len(),
                path
            )
            .expect("string write cannot fail");
        }
        WhyTarget::Decision(_) => {
            writeln!(summary, "Decision details:").expect("string write cannot fail");
        }
        WhyTarget::Line { file, line } => {
            writeln!(
                summary,
                "Found {} decision(s) that may affect `{}:{}`. Note: Line-level tracking requires git blame correlation.",
                decisions.len(),
                file,
                line
            )
            .expect("string write cannot fail");
        }
    }

    if let Some(first) = decisions.first() {
        writeln!(
            summary,
            "\nMost recent: \"{}\" by {} on {}.",
            first.summary,
            first.agent,
            first.timestamp.format("%Y-%m-%d")
        )
        .expect("string write cannot fail");
    }

    if let Some(last) = decisions.last()
        && decisions.len() > 1
    {
        writeln!(
            summary,
            "Earliest: \"{}\" by {} on {}.",
            last.summary,
            last.agent,
            last.timestamp.format("%Y-%m-%d")
        )
        .expect("string write cannot fail");
    }

    let agents: std::collections::HashSet<_> = decisions.iter().map(|d| d.agent.as_str()).collect();
    if agents.len() > 1 {
        writeln!(
            summary,
            "\nAgents involved: {}.",
            agents.into_iter().collect::<Vec<_>>().join(", ")
        )
        .expect("string write cannot fail");
    }

    summary
}

/// Retrieve a conversation excerpt around a decision point.
///
/// This is a placeholder that returns an empty excerpt.
/// Full implementation requires access to conversation storage.
pub fn get_conversation_excerpt(
    conversation_id: &str,
    message_id: &str,
    _context_messages: usize,
) -> ConversationExcerpt {
    ConversationExcerpt::new(conversation_id, message_id)
}

/// Retrieve conversation excerpts for decisions.
pub fn get_excerpts_for_decisions(
    decisions: &[DecisionEntry],
    context_messages: usize,
) -> Vec<ConversationExcerpt> {
    let mut seen_conversations = std::collections::HashSet::new();
    let mut excerpts = Vec::new();

    for decision in decisions {
        if seen_conversations.insert(decision.conversation_id.clone()) {
            excerpts.push(get_conversation_excerpt(
                &decision.conversation_id,
                &decision.message_id,
                context_messages,
            ));
        }
    }

    excerpts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::log::ChangeType;

    fn test_entry(conv_id: &str, summary: &str) -> DecisionEntry {
        DecisionEntry::new(
            conv_id,
            "msg-1",
            ChangeType::Create,
            summary,
            "reasoning",
            "coder",
        )
    }

    #[test]
    fn test_why_target_parse_file() {
        let target = WhyTarget::parse("src/lib.rs");
        assert_eq!(target, WhyTarget::File("src/lib.rs".to_string()));
    }

    #[test]
    fn test_why_target_parse_line() {
        let target = WhyTarget::parse("src/lib.rs:42");
        assert_eq!(
            target,
            WhyTarget::Line {
                file: "src/lib.rs".to_string(),
                line: 42
            }
        );
    }

    #[test]
    fn test_why_target_parse_invalid_line() {
        let target = WhyTarget::parse("src/lib.rs:abc");
        assert_eq!(target, WhyTarget::File("src/lib.rs:abc".to_string()));
    }

    #[test]
    fn test_redact_api_key() {
        let content = "api_key = 'sk-abc123def456'";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED"));
        assert!(!redacted.contains("sk-abc123def456"));
    }

    #[test]
    fn test_redact_password() {
        let content = "password: mysecretpassword";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED"));
        assert!(!redacted.contains("mysecretpassword"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let content = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_openai_key() {
        let content = "OPENAI_API_KEY=sk-proj-abcdefghijklmnopqrstuvwxyz";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_redact_github_token() {
        let content = "token: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_redact_aws_key() {
        let content = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let redacted = redact_sensitive(content);
        assert!(redacted.contains("[REDACTED"));
    }

    #[test]
    fn test_redact_preserves_normal_content() {
        let content = "This is normal content without secrets.";
        let redacted = redact_sensitive(content);
        assert_eq!(redacted, content);
    }

    #[test]
    fn test_why_file_with_decisions() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created module").with_file("src/lib.rs"));
        index.add_entry(test_entry("conv-2", "Fixed bug").with_file("src/lib.rs"));

        let explanation = why(&index, WhyTarget::File("src/lib.rs".to_string()));
        assert_eq!(explanation.decisions.len(), 2);
        assert!(explanation.summary.contains("2 decision(s)"));
    }

    #[test]
    fn test_why_file_no_decisions() {
        let index = DecisionIndex::new();
        let explanation = why(&index, WhyTarget::File("src/lib.rs".to_string()));
        assert!(explanation.decisions.is_empty());
        assert!(explanation.summary.contains("No recorded decisions"));
    }

    #[test]
    fn test_why_line() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created module").with_file("src/lib.rs"));

        let explanation = why(
            &index,
            WhyTarget::Line {
                file: "src/lib.rs".to_string(),
                line: 42,
            },
        );
        assert_eq!(explanation.target, "src/lib.rs:42");
        assert_eq!(explanation.decisions.len(), 1);
    }

    #[test]
    fn test_explanation_to_markdown() {
        let mut index = DecisionIndex::new();
        index.add_entry(test_entry("conv-1", "Created module").with_file("src/lib.rs"));

        let explanation = why(&index, WhyTarget::File("src/lib.rs".to_string()));
        let markdown = explanation.to_markdown();

        assert!(markdown.contains("# Why: src/lib.rs"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("## Decisions"));
        assert!(markdown.contains("Created module"));
    }

    #[test]
    fn test_conversation_excerpt_to_markdown() {
        let mut excerpt = ConversationExcerpt::new("conv-1", "msg-5");
        excerpt.messages_before.push(ExcerptMessage {
            role: "user".to_string(),
            content: "Can you create a module?".to_string(),
            timestamp: None,
        });
        excerpt.decision_message = Some(ExcerptMessage {
            role: "assistant".to_string(),
            content: "I'll create the module now.".to_string(),
            timestamp: None,
        });

        let markdown = excerpt.to_markdown();
        assert!(markdown.contains("**Conversation:** conv-1"));
        assert!(markdown.contains("**user:**"));
        assert!(markdown.contains("**assistant (decision):**"));
    }

    #[test]
    fn test_get_excerpts_for_decisions_deduplicates() {
        let decisions = vec![
            test_entry("conv-1", "First"),
            test_entry("conv-1", "Second"),
            test_entry("conv-2", "Third"),
        ];

        let excerpts = get_excerpts_for_decisions(&decisions, 2);
        assert_eq!(excerpts.len(), 2);
    }
}
