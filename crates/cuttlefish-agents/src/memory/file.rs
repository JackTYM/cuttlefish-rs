//! Memory file management for `.cuttlefish/memory.md`.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Maximum memory file size (1MB).
const MAX_MEMORY_SIZE: usize = 1024 * 1024;

/// Memory file error.
#[derive(Error, Debug)]
pub enum MemoryError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),
    /// File too large.
    #[error("Memory file exceeds {MAX_MEMORY_SIZE} bytes")]
    FileTooLarge,
}

/// A section in the memory file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySection {
    /// Project summary.
    Summary(String),
    /// Key decisions with date and rationale.
    KeyDecisions(Vec<DecisionItem>),
    /// Architecture components.
    Architecture(Vec<ArchitectureItem>),
    /// Gotchas and lessons learned.
    Gotchas(Vec<GotchaItem>),
    /// Rejected approaches.
    RejectedApproaches(Vec<RejectedItem>),
    /// Active context (current work).
    ActiveContext(ActiveContextData),
}

/// A key decision entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionItem {
    /// Date of the decision.
    pub date: String,
    /// The decision made.
    pub decision: String,
    /// Rationale for the decision.
    pub rationale: String,
}

/// An architecture component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureItem {
    /// Component name.
    pub component: String,
    /// Component description.
    pub description: String,
}

/// A gotcha or lesson learned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GotchaItem {
    /// The gotcha.
    pub gotcha: String,
    /// Context or explanation.
    pub context: String,
}

/// A rejected approach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedItem {
    /// The approach that was rejected.
    pub approach: String,
    /// Why it was rejected.
    pub reason: String,
}

/// Active context data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveContextData {
    /// Current task.
    pub current_task: Option<String>,
    /// Current blockers.
    pub blockers: Option<String>,
    /// Next steps.
    pub next_steps: Option<String>,
}

/// Project memory stored in `.cuttlefish/memory.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMemory {
    /// Project name.
    pub project_name: String,
    /// Summary section.
    pub summary: String,
    /// Key decisions.
    pub key_decisions: Vec<DecisionItem>,
    /// Architecture components.
    pub architecture: Vec<ArchitectureItem>,
    /// Gotchas and lessons.
    pub gotchas: Vec<GotchaItem>,
    /// Rejected approaches.
    pub rejected_approaches: Vec<RejectedItem>,
    /// Active context.
    pub active_context: ActiveContextData,
    /// Raw lines that couldn't be parsed (preserved for round-trip).
    raw_unknown_sections: HashMap<String, Vec<String>>,
}

impl ProjectMemory {
    /// Create a new empty project memory.
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            summary: String::new(),
            key_decisions: Vec::new(),
            architecture: Vec::new(),
            gotchas: Vec::new(),
            rejected_approaches: Vec::new(),
            active_context: ActiveContextData::default(),
            raw_unknown_sections: HashMap::new(),
        }
    }

    /// Load memory from a file path.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_MEMORY_SIZE as u64 {
            return Err(MemoryError::FileTooLarge);
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        Self::parse(reader)
    }

    /// Parse memory from a reader.
    pub fn parse<R: BufRead>(reader: R) -> Result<Self, MemoryError> {
        let mut memory = ProjectMemory::new("");
        let mut current_section: Option<String> = None;
        let mut section_lines: Vec<String> = Vec::new();

        for line_result in reader.lines() {
            let line = line_result?;

            if line.starts_with("# Project Memory:") {
                memory.project_name = line
                    .strip_prefix("# Project Memory:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                continue;
            }

            if let Some(section_header) = line.strip_prefix("## ") {
                if let Some(ref section_name) = current_section {
                    Self::process_section(&mut memory, section_name, &section_lines);
                }
                current_section = Some(section_header.trim().to_string());
                section_lines.clear();
                continue;
            }

            section_lines.push(line);
        }

        if let Some(ref section_name) = current_section {
            Self::process_section(&mut memory, section_name, &section_lines);
        }

        Ok(memory)
    }

    fn process_section(memory: &mut ProjectMemory, section_name: &str, lines: &[String]) {
        match section_name {
            "Summary" => {
                memory.summary = lines
                    .iter()
                    .map(|l| l.strip_prefix("> ").unwrap_or(l))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();
            }
            "Key Decisions" => {
                memory.key_decisions = Self::parse_decision_items(lines);
            }
            "Architecture" => {
                memory.architecture = Self::parse_architecture_items(lines);
            }
            "Gotchas & Lessons" => {
                memory.gotchas = Self::parse_gotcha_items(lines);
            }
            "Rejected Approaches" => {
                memory.rejected_approaches = Self::parse_rejected_items(lines);
            }
            "Active Context" => {
                memory.active_context = Self::parse_active_context(lines);
            }
            _ => {
                memory
                    .raw_unknown_sections
                    .insert(section_name.to_string(), lines.to_vec());
            }
        }
    }

    fn parse_decision_items(lines: &[String]) -> Vec<DecisionItem> {
        let mut items = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with("- **") {
                continue;
            }
            // Format: - **{date}**: {decision} — {rationale}
            if let Some(rest) = trimmed.strip_prefix("- **")
                && let Some(date_end) = rest.find("**:")
            {
                let date = rest[..date_end].to_string();
                let after_date = &rest[date_end + 3..].trim_start();
                // Split on " — " (em dash with spaces)
                let (decision, rationale) = if let Some(dash_pos) = after_date.find(" — ") {
                    (
                        after_date[..dash_pos].to_string(),
                        after_date[dash_pos + 5..].to_string(),
                    )
                } else {
                    (after_date.to_string(), String::new())
                };
                items.push(DecisionItem {
                    date,
                    decision,
                    rationale,
                });
            }
        }
        items
    }

    fn parse_architecture_items(lines: &[String]) -> Vec<ArchitectureItem> {
        let mut items = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                continue;
            }
            let content = &trimmed[2..];
            // Format: {component}: {description}
            if let Some(colon_pos) = content.find(": ") {
                items.push(ArchitectureItem {
                    component: content[..colon_pos].to_string(),
                    description: content[colon_pos + 2..].to_string(),
                });
            }
        }
        items
    }

    fn parse_gotcha_items(lines: &[String]) -> Vec<GotchaItem> {
        let mut items = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                continue;
            }
            let content = &trimmed[2..];
            // Format: {gotcha}: {context}
            if let Some(colon_pos) = content.find(": ") {
                items.push(GotchaItem {
                    gotcha: content[..colon_pos].to_string(),
                    context: content[colon_pos + 2..].to_string(),
                });
            }
        }
        items
    }

    fn parse_rejected_items(lines: &[String]) -> Vec<RejectedItem> {
        let mut items = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                continue;
            }
            let content = &trimmed[2..];
            // Format: {approach}: {why rejected}
            if let Some(colon_pos) = content.find(": ") {
                items.push(RejectedItem {
                    approach: content[..colon_pos].to_string(),
                    reason: content[colon_pos + 2..].to_string(),
                });
            }
        }
        items
    }

    fn parse_active_context(lines: &[String]) -> ActiveContextData {
        let mut data = ActiveContextData::default();
        for line in lines {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- Currently working on: ") {
                data.current_task = Some(rest.to_string());
            } else if let Some(rest) = trimmed.strip_prefix("- Blockers: ") {
                data.blockers = Some(rest.to_string());
            } else if let Some(rest) = trimmed.strip_prefix("- Next steps: ") {
                data.next_steps = Some(rest.to_string());
            }
        }
        data
    }

    /// Serialize memory to markdown string.
    pub fn serialize(&self) -> String {
        let mut output = String::new();

        writeln!(output, "# Project Memory: {}", self.project_name)
            .expect("string write cannot fail");
        writeln!(output).expect("string write cannot fail");

        // Summary
        writeln!(output, "## Summary").expect("string write cannot fail");
        if self.summary.is_empty() {
            writeln!(
                output,
                "> One-paragraph project description and current state"
            )
            .expect("string write cannot fail");
        } else {
            for line in self.summary.lines() {
                writeln!(output, "> {line}").expect("string write cannot fail");
            }
        }
        writeln!(output).expect("string write cannot fail");

        // Key Decisions
        writeln!(output, "## Key Decisions").expect("string write cannot fail");
        if self.key_decisions.is_empty() {
            writeln!(output, "- **{{date}}**: {{decision}} — {{rationale}}")
                .expect("string write cannot fail");
        } else {
            for item in &self.key_decisions {
                if item.rationale.is_empty() {
                    writeln!(output, "- **{}**: {}", item.date, item.decision)
                        .expect("string write cannot fail");
                } else {
                    writeln!(
                        output,
                        "- **{}**: {} — {}",
                        item.date, item.decision, item.rationale
                    )
                    .expect("string write cannot fail");
                }
            }
        }
        writeln!(output).expect("string write cannot fail");

        // Architecture
        writeln!(output, "## Architecture").expect("string write cannot fail");
        if self.architecture.is_empty() {
            writeln!(output, "- {{component}}: {{description}}").expect("string write cannot fail");
        } else {
            for item in &self.architecture {
                writeln!(output, "- {}: {}", item.component, item.description)
                    .expect("string write cannot fail");
            }
        }
        writeln!(output).expect("string write cannot fail");

        // Gotchas & Lessons
        writeln!(output, "## Gotchas & Lessons").expect("string write cannot fail");
        if self.gotchas.is_empty() {
            writeln!(output, "- {{gotcha}}: {{context}}").expect("string write cannot fail");
        } else {
            for item in &self.gotchas {
                writeln!(output, "- {}: {}", item.gotcha, item.context)
                    .expect("string write cannot fail");
            }
        }
        writeln!(output).expect("string write cannot fail");

        // Rejected Approaches
        writeln!(output, "## Rejected Approaches").expect("string write cannot fail");
        if self.rejected_approaches.is_empty() {
            writeln!(output, "- {{approach}}: {{why rejected}}").expect("string write cannot fail");
        } else {
            for item in &self.rejected_approaches {
                writeln!(output, "- {}: {}", item.approach, item.reason)
                    .expect("string write cannot fail");
            }
        }
        writeln!(output).expect("string write cannot fail");

        // Active Context
        writeln!(output, "## Active Context").expect("string write cannot fail");
        writeln!(
            output,
            "- Currently working on: {}",
            self.active_context
                .current_task
                .as_deref()
                .unwrap_or("{task}")
        )
        .expect("string write cannot fail");
        writeln!(
            output,
            "- Blockers: {}",
            self.active_context
                .blockers
                .as_deref()
                .unwrap_or("{blockers}")
        )
        .expect("string write cannot fail");
        writeln!(
            output,
            "- Next steps: {}",
            self.active_context
                .next_steps
                .as_deref()
                .unwrap_or("{steps}")
        )
        .expect("string write cannot fail");

        // Unknown sections (preserved for round-trip)
        for (section_name, lines) in &self.raw_unknown_sections {
            writeln!(output).expect("string write cannot fail");
            writeln!(output, "## {section_name}").expect("string write cannot fail");
            for line in lines {
                writeln!(output, "{line}").expect("string write cannot fail");
            }
        }

        output
    }

    /// Save memory to a file path.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), MemoryError> {
        let content = self.serialize();
        if content.len() > MAX_MEMORY_SIZE {
            return Err(MemoryError::FileTooLarge);
        }

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Get the default memory file path for a project root.
    pub fn default_path(project_root: impl AsRef<Path>) -> PathBuf {
        project_root.as_ref().join(".cuttlefish").join("memory.md")
    }

    /// Load or create memory for a project.
    pub fn load_or_create(
        project_root: impl AsRef<Path>,
        project_name: impl Into<String>,
    ) -> Result<Self, MemoryError> {
        let path = Self::default_path(&project_root);
        if path.exists() {
            Self::load(&path)
        } else {
            Ok(Self::new(project_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_new_memory() {
        let memory = ProjectMemory::new("test-project");
        assert_eq!(memory.project_name, "test-project");
        assert!(memory.summary.is_empty());
        assert!(memory.key_decisions.is_empty());
    }

    #[test]
    fn test_serialize_empty_memory() {
        let memory = ProjectMemory::new("test-project");
        let output = memory.serialize();
        assert!(output.contains("# Project Memory: test-project"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("## Key Decisions"));
        assert!(output.contains("## Architecture"));
        assert!(output.contains("## Gotchas & Lessons"));
        assert!(output.contains("## Rejected Approaches"));
        assert!(output.contains("## Active Context"));
    }

    #[test]
    fn test_parse_and_serialize_roundtrip() {
        let mut memory = ProjectMemory::new("roundtrip-test");
        memory.summary = "This is a test project for memory round-trip.".to_string();
        memory.key_decisions.push(DecisionItem {
            date: "2024-01-15".to_string(),
            decision: "Use Rust".to_string(),
            rationale: "Memory safety".to_string(),
        });
        memory.architecture.push(ArchitectureItem {
            component: "memory".to_string(),
            description: "Handles project memory".to_string(),
        });
        memory.gotchas.push(GotchaItem {
            gotcha: "UTF-8 encoding".to_string(),
            context: "Always use UTF-8 for memory files".to_string(),
        });
        memory.rejected_approaches.push(RejectedItem {
            approach: "JSON format".to_string(),
            reason: "Not human-readable enough".to_string(),
        });
        memory.active_context = ActiveContextData {
            current_task: Some("Implementing memory system".to_string()),
            blockers: Some("None".to_string()),
            next_steps: Some("Add tests".to_string()),
        };

        let serialized = memory.serialize();
        let cursor = Cursor::new(serialized.as_bytes());
        let parsed = ProjectMemory::parse(cursor).expect("parse should succeed");

        assert_eq!(parsed.project_name, memory.project_name);
        assert_eq!(parsed.summary, memory.summary);
        assert_eq!(parsed.key_decisions.len(), memory.key_decisions.len());
        assert_eq!(parsed.key_decisions[0].date, "2024-01-15");
        assert_eq!(parsed.key_decisions[0].decision, "Use Rust");
        assert_eq!(parsed.key_decisions[0].rationale, "Memory safety");
        assert_eq!(parsed.architecture.len(), memory.architecture.len());
        assert_eq!(parsed.gotchas.len(), memory.gotchas.len());
        assert_eq!(
            parsed.rejected_approaches.len(),
            memory.rejected_approaches.len()
        );
        assert_eq!(
            parsed.active_context.current_task,
            memory.active_context.current_task
        );
    }

    #[test]
    fn test_parse_decision_without_rationale() {
        let content = r#"# Project Memory: test

## Key Decisions
- **2024-01-15**: Use Rust
"#;
        let cursor = Cursor::new(content.as_bytes());
        let memory = ProjectMemory::parse(cursor).expect("parse should succeed");
        assert_eq!(memory.key_decisions.len(), 1);
        assert_eq!(memory.key_decisions[0].decision, "Use Rust");
        assert!(memory.key_decisions[0].rationale.is_empty());
    }

    #[test]
    fn test_unknown_section_preserved() {
        let content = r#"# Project Memory: test

## Summary
> Test summary

## Custom Section
Some custom content
More content
"#;
        let cursor = Cursor::new(content.as_bytes());
        let memory = ProjectMemory::parse(cursor).expect("parse should succeed");
        assert!(memory.raw_unknown_sections.contains_key("Custom Section"));

        let serialized = memory.serialize();
        assert!(serialized.contains("## Custom Section"));
        assert!(serialized.contains("Some custom content"));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let path = temp_dir.path().join(".cuttlefish").join("memory.md");

        let mut memory = ProjectMemory::new("save-test");
        memory.summary = "Test save and load".to_string();
        memory.save(&path).expect("save should succeed");

        let loaded = ProjectMemory::load(&path).expect("load should succeed");
        assert_eq!(loaded.project_name, "save-test");
        assert_eq!(loaded.summary, "Test save and load");
    }

    #[test]
    fn test_load_or_create_new() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let memory =
            ProjectMemory::load_or_create(temp_dir.path(), "new-project").expect("should succeed");
        assert_eq!(memory.project_name, "new-project");
    }

    #[test]
    fn test_load_or_create_existing() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let path = ProjectMemory::default_path(temp_dir.path());

        let mut original = ProjectMemory::new("existing-project");
        original.summary = "Existing summary".to_string();
        original.save(&path).expect("save should succeed");

        let loaded =
            ProjectMemory::load_or_create(temp_dir.path(), "ignored-name").expect("should succeed");
        assert_eq!(loaded.project_name, "existing-project");
        assert_eq!(loaded.summary, "Existing summary");
    }

    #[test]
    fn test_multiline_summary() {
        let content = r#"# Project Memory: test

## Summary
> Line one
> Line two
> Line three
"#;
        let cursor = Cursor::new(content.as_bytes());
        let memory = ProjectMemory::parse(cursor).expect("parse should succeed");
        assert!(memory.summary.contains("Line one"));
        assert!(memory.summary.contains("Line two"));
        assert!(memory.summary.contains("Line three"));
    }
}
