//! Discord message formatting utilities.

/// Maximum Discord message length.
pub const DISCORD_MESSAGE_LIMIT: usize = 2000;

/// Format a code snippet for Discord (wrapped in backticks).
pub fn format_code_block(code: &str, language: &str) -> String {
    format!("```{}\n{}\n```", language, code)
}

/// Format a status embed as text (for channels without embed support).
pub fn format_status(status: &str, message: &str) -> String {
    let emoji = match status {
        "success" | "ok" => "✅",
        "error" | "failure" => "❌",
        "running" | "in_progress" => "⏳",
        "warning" => "⚠️",
        _ => "ℹ️",
    };
    format!("{} **{}**: {}", emoji, status.to_uppercase(), message)
}

/// Split a long message into Discord-safe chunks.
pub fn split_message(text: &str) -> Vec<String> {
    if text.len() <= DISCORD_MESSAGE_LIMIT {
        return vec![text.to_string()];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(DISCORD_MESSAGE_LIMIT)
        .map(|c| c.iter().collect())
        .collect()
}

/// Format a diff for Discord with code block.
pub fn format_diff(patch: &str) -> Vec<String> {
    let formatted = format_code_block(patch, "diff");
    split_message(&formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_code_block() {
        let block = format_code_block("fn main() {}", "rust");
        assert!(block.starts_with("```rust"));
        assert!(block.ends_with("```"));
    }

    #[test]
    fn test_format_status_success() {
        let s = format_status("success", "Build passed");
        assert!(s.contains("✅"));
        assert!(s.contains("Build passed"));
    }

    #[test]
    fn test_split_message_short() {
        let parts = split_message("short");
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_split_message_long() {
        let long = "x".repeat(5000);
        let parts = split_message(&long);
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.len() <= DISCORD_MESSAGE_LIMIT));
    }
}
