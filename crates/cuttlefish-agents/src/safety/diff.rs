//! Diff generation engine for file change previews.
//!
//! This module provides unified diff generation for comparing file contents,
//! with support for syntax highlighting hints and summary statistics.

use std::fmt;
use std::path::Path;

/// Maximum file size (in bytes) for diff generation.
/// Files larger than this will show a warning instead of a diff.
pub const MAX_DIFF_FILE_SIZE: usize = 1024 * 1024; // 1MB

/// A complete diff between two versions of a file.
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path to the file being diffed.
    pub path: String,
    /// Original content (None for new files).
    pub old_content: Option<String>,
    /// New content.
    pub new_content: String,
    /// Diff hunks showing the changes.
    pub hunks: Vec<DiffHunk>,
    /// Summary statistics.
    pub stats: DiffStats,
    /// Detected language for syntax highlighting.
    pub language: Option<String>,
}

impl FileDiff {
    /// Generate a diff between old and new content.
    pub fn generate(path: impl Into<String>, old_content: Option<&str>, new_content: &str) -> Self {
        let path = path.into();
        let language = detect_language(&path);

        let old_lines: Vec<&str> = old_content.map(|s| s.lines().collect()).unwrap_or_default();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let hunks = compute_hunks(&old_lines, &new_lines);
        let stats = compute_stats(&hunks);

        Self {
            path,
            old_content: old_content.map(String::from),
            new_content: new_content.to_string(),
            hunks,
            stats,
            language,
        }
    }

    /// Generate a diff for a new file (no old content).
    pub fn for_new_file(path: impl Into<String>, content: &str) -> Self {
        Self::generate(path, None, content)
    }

    /// Generate a diff for file deletion.
    pub fn for_deletion(path: impl Into<String>, old_content: &str) -> Self {
        Self::generate(path, Some(old_content), "")
    }

    /// Check if this diff represents a new file.
    pub fn is_new_file(&self) -> bool {
        self.old_content.is_none()
    }

    /// Check if this diff represents a file deletion.
    pub fn is_deletion(&self) -> bool {
        self.old_content.is_some() && self.new_content.is_empty()
    }

    /// Check if the file is too large for detailed diff.
    pub fn is_too_large(&self) -> bool {
        self.old_content.as_ref().map(|s| s.len()).unwrap_or(0) > MAX_DIFF_FILE_SIZE
            || self.new_content.len() > MAX_DIFF_FILE_SIZE
    }

    /// Check if this is a binary file (contains null bytes).
    pub fn is_binary(&self) -> bool {
        self.old_content
            .as_ref()
            .map(|s| s.contains('\0'))
            .unwrap_or(false)
            || self.new_content.contains('\0')
    }

    /// Render the diff in unified diff format.
    pub fn to_unified_diff(&self) -> String {
        let mut output = String::new();

        // Header
        let old_path = if self.is_new_file() {
            "/dev/null".to_string()
        } else {
            format!("a/{}", self.path)
        };
        let new_path = if self.is_deletion() {
            "/dev/null".to_string()
        } else {
            format!("b/{}", self.path)
        };

        output.push_str(&format!("--- {old_path}\n"));
        output.push_str(&format!("+++ {new_path}\n"));

        // Hunks
        for hunk in &self.hunks {
            output.push_str(&hunk.to_unified_format());
        }

        output
    }
}

impl fmt::Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_unified_diff())
    }
}

/// A hunk in a diff, representing a contiguous region of changes.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Starting line number in the old file (1-indexed).
    pub old_start: usize,
    /// Number of lines from the old file in this hunk.
    pub old_lines: usize,
    /// Starting line number in the new file (1-indexed).
    pub new_start: usize,
    /// Number of lines from the new file in this hunk.
    pub new_lines: usize,
    /// The lines in this hunk.
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Render this hunk in unified diff format.
    pub fn to_unified_format(&self) -> String {
        let mut output = String::new();

        // Hunk header
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            self.old_start, self.old_lines, self.new_start, self.new_lines
        ));

        // Lines
        for line in &self.lines {
            output.push_str(&line.to_unified_format());
            output.push('\n');
        }

        output
    }
}

/// A single line in a diff.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    /// Context line (unchanged).
    Context {
        /// The line content.
        content: String,
        /// Line number in old file.
        old_line: usize,
        /// Line number in new file.
        new_line: usize,
    },
    /// Added line.
    Added {
        /// The line content.
        content: String,
        /// Line number in new file.
        new_line: usize,
    },
    /// Removed line.
    Removed {
        /// The line content.
        content: String,
        /// Line number in old file.
        old_line: usize,
    },
}

impl DiffLine {
    /// Get the change type of this line.
    pub fn change_type(&self) -> ChangeType {
        match self {
            Self::Context { .. } => ChangeType::Context,
            Self::Added { .. } => ChangeType::Added,
            Self::Removed { .. } => ChangeType::Removed,
        }
    }

    /// Get the content of this line.
    pub fn content(&self) -> &str {
        match self {
            Self::Context { content, .. }
            | Self::Added { content, .. }
            | Self::Removed { content, .. } => content,
        }
    }

    /// Render this line in unified diff format.
    pub fn to_unified_format(&self) -> String {
        match self {
            Self::Context { content, .. } => format!(" {content}"),
            Self::Added { content, .. } => format!("+{content}"),
            Self::Removed { content, .. } => format!("-{content}"),
        }
    }
}

/// Type of change for a diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Line is unchanged (context).
    Context,
    /// Line was added.
    Added,
    /// Line was removed.
    Removed,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context => write!(f, "context"),
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
        }
    }
}

/// Summary statistics for a diff.
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    /// Number of lines added.
    pub lines_added: usize,
    /// Number of lines removed.
    pub lines_removed: usize,
    /// Number of hunks.
    pub hunks: usize,
}

impl DiffStats {
    /// Get the total number of changed lines.
    pub fn total_changes(&self) -> usize {
        self.lines_added + self.lines_removed
    }

    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        self.lines_added > 0 || self.lines_removed > 0
    }
}

impl fmt::Display for DiffStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "+{} -{} ({} hunks)",
            self.lines_added, self.lines_removed, self.hunks
        )
    }
}

/// Detect the programming language from a file path.
pub fn detect_language(path: &str) -> Option<String> {
    let extension = Path::new(path).extension()?.to_str()?;

    let language = match extension.to_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" => "bash",
        "zsh" => "zsh",
        "fish" => "fish",
        "ps1" => "powershell",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "less" => "less",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" | "markdown" => "markdown",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "lua" => "lua",
        "r" => "r",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "fs" | "fsi" | "fsx" => "fsharp",
        "clj" | "cljs" | "cljc" => "clojure",
        "vue" => "vue",
        "svelte" => "svelte",
        _ => return None,
    };

    Some(language.to_string())
}

// Compute diff hunks using a simple LCS-based algorithm
fn compute_hunks(old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffHunk> {
    let ops = compute_edit_operations(old_lines, new_lines);

    if ops.is_empty() {
        return Vec::new();
    }

    let mut hunks = Vec::new();
    let mut current_hunk: Option<HunkBuilder> = None;
    let context_lines = 3;

    let mut old_idx = 0usize;
    let mut new_idx = 0usize;

    for op in &ops {
        match op {
            EditOp::Equal => {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.add_context(old_lines[old_idx], old_idx + 1, new_idx + 1);
                    if hunk.trailing_context >= context_lines {
                        hunks.push(current_hunk.take().expect("hunk exists").build());
                    }
                }
                old_idx += 1;
                new_idx += 1;
            }
            EditOp::Insert => {
                let hunk = current_hunk.get_or_insert_with(|| {
                    let start_old = old_idx.saturating_sub(context_lines);
                    let start_new = new_idx.saturating_sub(context_lines);
                    let mut builder = HunkBuilder::new(start_old + 1, start_new + 1);
                    // Add leading context
                    for i in start_old..old_idx {
                        if i < old_lines.len() {
                            let ctx_new = start_new + (i - start_old);
                            builder.add_context(old_lines[i], i + 1, ctx_new + 1);
                        }
                    }
                    builder
                });
                hunk.add_added(new_lines[new_idx], new_idx + 1);
                new_idx += 1;
            }
            EditOp::Delete => {
                let hunk = current_hunk.get_or_insert_with(|| {
                    let start_old = old_idx.saturating_sub(context_lines);
                    let start_new = new_idx.saturating_sub(context_lines);
                    let mut builder = HunkBuilder::new(start_old + 1, start_new + 1);
                    // Add leading context
                    for i in start_old..old_idx {
                        if i < old_lines.len() {
                            let ctx_new = start_new + (i - start_old);
                            builder.add_context(old_lines[i], i + 1, ctx_new + 1);
                        }
                    }
                    builder
                });
                hunk.add_removed(old_lines[old_idx], old_idx + 1);
                old_idx += 1;
            }
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk.build());
    }

    hunks
}

#[derive(Debug, Clone, Copy)]
enum EditOp {
    Equal,
    Insert,
    Delete,
}

fn compute_edit_operations(old: &[&str], new: &[&str]) -> Vec<EditOp> {
    let m = old.len();
    let n = new.len();

    if m == 0 && n == 0 {
        return Vec::new();
    }

    if m == 0 {
        return vec![EditOp::Insert; n];
    }

    if n == 0 {
        return vec![EditOp::Delete; m];
    }

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to get operations
    let mut ops = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            ops.push(EditOp::Equal);
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(EditOp::Insert);
            j -= 1;
        } else {
            ops.push(EditOp::Delete);
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

struct HunkBuilder {
    old_start: usize,
    new_start: usize,
    lines: Vec<DiffLine>,
    old_count: usize,
    new_count: usize,
    trailing_context: usize,
}

impl HunkBuilder {
    fn new(old_start: usize, new_start: usize) -> Self {
        Self {
            old_start,
            new_start,
            lines: Vec::new(),
            old_count: 0,
            new_count: 0,
            trailing_context: 0,
        }
    }

    fn add_context(&mut self, content: &str, old_line: usize, new_line: usize) {
        self.lines.push(DiffLine::Context {
            content: content.to_string(),
            old_line,
            new_line,
        });
        self.old_count += 1;
        self.new_count += 1;
        self.trailing_context += 1;
    }

    fn add_added(&mut self, content: &str, new_line: usize) {
        self.lines.push(DiffLine::Added {
            content: content.to_string(),
            new_line,
        });
        self.new_count += 1;
        self.trailing_context = 0;
    }

    fn add_removed(&mut self, content: &str, old_line: usize) {
        self.lines.push(DiffLine::Removed {
            content: content.to_string(),
            old_line,
        });
        self.old_count += 1;
        self.trailing_context = 0;
    }

    fn build(mut self) -> DiffHunk {
        // Trim excess trailing context
        while self.trailing_context > 3 && !self.lines.is_empty() {
            if matches!(self.lines.last(), Some(DiffLine::Context { .. })) {
                self.lines.pop();
                self.old_count -= 1;
                self.new_count -= 1;
                self.trailing_context -= 1;
            } else {
                break;
            }
        }

        DiffHunk {
            old_start: self.old_start,
            old_lines: self.old_count,
            new_start: self.new_start,
            new_lines: self.new_count,
            lines: self.lines,
        }
    }
}

fn compute_stats(hunks: &[DiffHunk]) -> DiffStats {
    let mut stats = DiffStats {
        hunks: hunks.len(),
        ..Default::default()
    };

    for hunk in hunks {
        for line in &hunk.lines {
            match line {
                DiffLine::Added { .. } => stats.lines_added += 1,
                DiffLine::Removed { .. } => stats.lines_removed += 1,
                DiffLine::Context { .. } => {}
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_new_file() {
        let diff = FileDiff::for_new_file("test.rs", "fn main() {\n    println!(\"Hello\");\n}");

        assert!(diff.is_new_file());
        assert!(!diff.is_deletion());
        assert_eq!(diff.stats.lines_added, 3);
        assert_eq!(diff.stats.lines_removed, 0);
        assert_eq!(diff.language, Some("rust".to_string()));
    }

    #[test]
    fn test_diff_deletion() {
        let diff = FileDiff::for_deletion("test.py", "print('hello')\nprint('world')");

        assert!(!diff.is_new_file());
        assert!(diff.is_deletion());
        assert_eq!(diff.stats.lines_added, 0);
        assert_eq!(diff.stats.lines_removed, 2);
    }

    #[test]
    fn test_diff_modification() {
        let old = "hello\nworld";
        let new = "hello\nrust\nworld";

        let diff = FileDiff::generate("test.txt", Some(old), new);

        assert!(!diff.is_new_file());
        assert!(!diff.is_deletion());
        assert_eq!(diff.stats.lines_added, 1);
        assert_eq!(diff.stats.lines_removed, 0);
    }

    #[test]
    fn test_diff_replacement() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";

        let diff = FileDiff::generate("test.txt", Some(old), new);

        assert_eq!(diff.stats.lines_added, 1);
        assert_eq!(diff.stats.lines_removed, 1);
    }

    #[test]
    fn test_diff_no_changes() {
        let content = "same\ncontent";
        let diff = FileDiff::generate("test.txt", Some(content), content);

        assert!(!diff.stats.has_changes());
        assert_eq!(diff.stats.total_changes(), 0);
    }

    #[test]
    fn test_unified_diff_format() {
        let old = "hello\nworld";
        let new = "hello\nrust";

        let diff = FileDiff::generate("test.txt", Some(old), new);
        let unified = diff.to_unified_diff();

        assert!(unified.contains("--- a/test.txt"));
        assert!(unified.contains("+++ b/test.txt"));
        assert!(unified.contains("-world"));
        assert!(unified.contains("+rust"));
    }

    #[test]
    fn test_diff_line_types() {
        let context = DiffLine::Context {
            content: "test".to_string(),
            old_line: 1,
            new_line: 1,
        };
        assert_eq!(context.change_type(), ChangeType::Context);
        assert_eq!(context.content(), "test");
        assert_eq!(context.to_unified_format(), " test");

        let added = DiffLine::Added {
            content: "new".to_string(),
            new_line: 2,
        };
        assert_eq!(added.change_type(), ChangeType::Added);
        assert_eq!(added.to_unified_format(), "+new");

        let removed = DiffLine::Removed {
            content: "old".to_string(),
            old_line: 1,
        };
        assert_eq!(removed.change_type(), ChangeType::Removed);
        assert_eq!(removed.to_unified_format(), "-old");
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(detect_language("script.py"), Some("python".to_string()));
        assert_eq!(detect_language("app.tsx"), Some("tsx".to_string()));
        assert_eq!(detect_language("config.yaml"), Some("yaml".to_string()));
        assert_eq!(detect_language("Cargo.toml"), Some("toml".to_string()));
        assert_eq!(detect_language("unknown.xyz"), None);
    }

    #[test]
    fn test_diff_stats_display() {
        let stats = DiffStats {
            lines_added: 10,
            lines_removed: 5,
            hunks: 2,
        };

        let display = format!("{stats}");
        assert!(display.contains("+10"));
        assert!(display.contains("-5"));
        assert!(display.contains("2 hunks"));
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(format!("{}", ChangeType::Context), "context");
        assert_eq!(format!("{}", ChangeType::Added), "added");
        assert_eq!(format!("{}", ChangeType::Removed), "removed");
    }

    #[test]
    fn test_binary_detection() {
        let diff = FileDiff::generate("test.bin", Some("hello\0world"), "new content");
        assert!(diff.is_binary());

        let diff = FileDiff::generate("test.txt", Some("hello world"), "new content");
        assert!(!diff.is_binary());
    }

    #[test]
    fn test_large_file_detection() {
        let large_content = "x".repeat(MAX_DIFF_FILE_SIZE + 1);
        let diff = FileDiff::generate("large.txt", Some(&large_content), "small");
        assert!(diff.is_too_large());

        let small_content = "small content";
        let diff = FileDiff::generate("small.txt", Some(small_content), "also small");
        assert!(!diff.is_too_large());
    }

    #[test]
    fn test_hunk_unified_format() {
        let hunk = DiffHunk {
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 4,
            lines: vec![
                DiffLine::Context {
                    content: "line1".to_string(),
                    old_line: 1,
                    new_line: 1,
                },
                DiffLine::Added {
                    content: "new line".to_string(),
                    new_line: 2,
                },
                DiffLine::Context {
                    content: "line2".to_string(),
                    old_line: 2,
                    new_line: 3,
                },
            ],
        };

        let format = hunk.to_unified_format();
        assert!(format.contains("@@ -1,3 +1,4 @@"));
        assert!(format.contains(" line1"));
        assert!(format.contains("+new line"));
    }

    #[test]
    fn test_empty_diff() {
        let diff = FileDiff::generate("empty.txt", Some(""), "");
        assert!(!diff.stats.has_changes());
        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn test_multiline_insertion() {
        let old = "line1\nline3";
        let new = "line1\nline2a\nline2b\nline3";

        let diff = FileDiff::generate("test.txt", Some(old), new);
        assert_eq!(diff.stats.lines_added, 2);
        assert_eq!(diff.stats.lines_removed, 0);
    }

    #[test]
    fn test_multiline_deletion() {
        let old = "line1\nline2a\nline2b\nline3";
        let new = "line1\nline3";

        let diff = FileDiff::generate("test.txt", Some(old), new);
        assert_eq!(diff.stats.lines_added, 0);
        assert_eq!(diff.stats.lines_removed, 2);
    }
}
