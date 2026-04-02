//! Hashline: line-anchored file editing with content-addressable hashes.
//!
//! Each line in a file gets a short hash (5 hex chars) computed from its content.
//! Edits reference lines by hash, enabling precise surgical modifications without
//! line number drift or search ambiguity.

use std::collections::HashMap;

const HASH_SEED: u64 = 0x6e52736ac806831e;

fn xxhash64_simple(data: &[u8], seed: u64) -> u64 {
    const PRIME1: u64 = 0x9E3779B185EBCA87;
    const PRIME2: u64 = 0xC2B2AE3D27D4EB4F;
    const PRIME3: u64 = 0x165667B19E3779F9;
    const PRIME4: u64 = 0x85EBCA77C2B2AE63;
    const PRIME5: u64 = 0x27D4EB2F165667C5;

    let mut h: u64 = seed.wrapping_add(PRIME5).wrapping_add(data.len() as u64);

    for chunk in data.chunks(8) {
        if chunk.len() == 8 {
            let k = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            h ^= k.wrapping_mul(PRIME2).rotate_left(31).wrapping_mul(PRIME1);
            h = h.rotate_left(27).wrapping_mul(PRIME1).wrapping_add(PRIME4);
        } else {
            for &b in chunk {
                h ^= (b as u64).wrapping_mul(PRIME5);
                h = h.rotate_left(11).wrapping_mul(PRIME1);
            }
        }
    }

    h ^= h >> 33;
    h = h.wrapping_mul(PRIME2);
    h ^= h >> 29;
    h = h.wrapping_mul(PRIME3);
    h ^= h >> 32;
    h
}

/// Compute a 5-character hex hash for a line of content.
pub fn line_hash(content: &str) -> String {
    let h = xxhash64_simple(content.trim_end().as_bytes(), HASH_SEED);
    format!("{:05x}", h & 0xFFFFF)
}

/// A line with its hash and content.
#[derive(Debug, Clone, PartialEq)]
pub struct HashedLine {
    /// Line number (1-indexed).
    pub line_num: usize,
    /// 5-char hex hash of the line content.
    pub hash: String,
    /// The actual line content (without trailing newline).
    pub content: String,
}

/// Parse file content into hashed lines.
pub fn hash_file_lines(content: &str) -> Vec<HashedLine> {
    content
        .lines()
        .enumerate()
        .map(|(i, line)| HashedLine {
            line_num: i + 1,
            hash: line_hash(line),
            content: line.to_string(),
        })
        .collect()
}

/// Format file with line hashes for display to the model.
pub fn format_with_hashes(content: &str) -> String {
    hash_file_lines(content)
        .into_iter()
        .map(|hl| format!("{} | {}", hl.hash, hl.content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a lookup map from hash to line indices (handles hash collisions).
pub fn build_hash_index(lines: &[HashedLine]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        index.entry(line.hash.clone()).or_default().push(i);
    }
    index
}

/// Error type for edit operations.
#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// Hash not found in file.
    HashNotFound(String),
    /// Multiple lines match the hash (collision).
    AmbiguousHash {
        /// The ambiguous hash value.
        hash: String,
        /// Number of lines matching this hash.
        count: usize,
    },
    /// Content at hash doesn't match expected.
    ContentMismatch {
        /// The hash where mismatch occurred.
        hash: String,
        /// What the caller expected to find.
        expected: String,
        /// What was actually in the file.
        actual: String,
    },
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::HashNotFound(h) => write!(f, "Hash '{}' not found in file", h),
            EditError::AmbiguousHash { hash, count } => {
                write!(f, "Hash '{}' matches {} lines (ambiguous)", hash, count)
            }
            EditError::ContentMismatch {
                hash,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Content mismatch at hash '{}': expected '{}', found '{}'",
                    hash, expected, actual
                )
            }
        }
    }
}

impl std::error::Error for EditError {}

/// A single edit operation targeting a line by hash.
#[derive(Debug, Clone)]
pub struct LineEdit {
    /// Hash of the line to edit.
    pub hash: String,
    /// Expected content (for verification, optional).
    pub expected_content: Option<String>,
    /// New content to replace with. None = delete the line.
    pub new_content: Option<String>,
}

/// Apply a series of edits to file content.
pub fn apply_edits(content: &str, edits: &[LineEdit]) -> Result<String, EditError> {
    let mut lines = hash_file_lines(content);
    let mut index = build_hash_index(&lines);

    for edit in edits {
        let indices = index.get(&edit.hash).cloned().unwrap_or_default();

        if indices.is_empty() {
            return Err(EditError::HashNotFound(edit.hash.clone()));
        }
        if indices.len() > 1 {
            return Err(EditError::AmbiguousHash {
                hash: edit.hash.clone(),
                count: indices.len(),
            });
        }

        let idx = indices[0];
        let line = &lines[idx];

        if let Some(expected) = &edit.expected_content
            && line.content.trim() != expected.trim()
        {
            return Err(EditError::ContentMismatch {
                hash: edit.hash.clone(),
                expected: expected.clone(),
                actual: line.content.clone(),
            });
        }

        match &edit.new_content {
            Some(new) => {
                let old_hash = lines[idx].hash.clone();
                lines[idx].content = new.clone();
                lines[idx].hash = line_hash(new);

                if let Some(v) = index.get_mut(&old_hash) {
                    v.retain(|&i| i != idx);
                }
                index.entry(lines[idx].hash.clone()).or_default().push(idx);
            }
            None => {
                let old_hash = lines[idx].hash.clone();
                lines[idx].content = String::new();
                lines[idx].hash = line_hash("");

                if let Some(v) = index.get_mut(&old_hash) {
                    v.retain(|&i| i != idx);
                }
            }
        }
    }

    let result: Vec<&str> = lines
        .iter()
        .filter(|l| !l.content.is_empty() || l.hash == line_hash(""))
        .map(|l| l.content.as_str())
        .collect();

    Ok(result.join("\n"))
}

/// Insert new lines after a line identified by hash.
pub fn insert_after(
    content: &str,
    after_hash: &str,
    new_lines: &[String],
) -> Result<String, EditError> {
    let lines = hash_file_lines(content);
    let index = build_hash_index(&lines);

    let indices = index.get(after_hash).cloned().unwrap_or_default();
    if indices.is_empty() {
        return Err(EditError::HashNotFound(after_hash.to_string()));
    }
    if indices.len() > 1 {
        return Err(EditError::AmbiguousHash {
            hash: after_hash.to_string(),
            count: indices.len(),
        });
    }

    let insert_idx = indices[0] + 1;
    let mut result: Vec<String> = lines.iter().map(|l| l.content.clone()).collect();
    for (i, line) in new_lines.iter().enumerate() {
        result.insert(insert_idx + i, line.clone());
    }

    Ok(result.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_hash_deterministic() {
        let h1 = line_hash("fn main() {");
        let h2 = line_hash("fn main() {");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 5);
    }

    #[test]
    fn test_line_hash_different_content() {
        let h1 = line_hash("fn main() {");
        let h2 = line_hash("fn foo() {");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_file_lines() {
        let content = "line one\nline two\nline three";
        let lines = hash_file_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].line_num, 1);
        assert_eq!(lines[1].line_num, 2);
        assert_eq!(lines[2].line_num, 3);
    }

    #[test]
    fn test_format_with_hashes() {
        let content = "hello\nworld";
        let formatted = format_with_hashes(content);
        assert!(formatted.contains(" | hello"));
        assert!(formatted.contains(" | world"));
    }

    #[test]
    fn test_apply_edit_replace() {
        let content = "fn main() {\n    println!(\"hello\");\n}";
        let lines = hash_file_lines(content);
        let hello_hash = lines[1].hash.clone();

        let edits = vec![LineEdit {
            hash: hello_hash,
            expected_content: Some("    println!(\"hello\");".to_string()),
            new_content: Some("    println!(\"world\");".to_string()),
        }];

        let result = apply_edits(content, &edits).unwrap();
        assert!(result.contains("world"));
        assert!(!result.contains("hello"));
    }

    #[test]
    fn test_apply_edit_hash_not_found() {
        let content = "line one\nline two";
        let edits = vec![LineEdit {
            hash: "fffff".to_string(),
            expected_content: None,
            new_content: Some("new".to_string()),
        }];

        let result = apply_edits(content, &edits);
        assert!(matches!(result, Err(EditError::HashNotFound(_))));
    }

    #[test]
    fn test_insert_after() {
        let content = "line one\nline two\nline three";
        let lines = hash_file_lines(content);
        let hash = lines[0].hash.clone();

        let result = insert_after(content, &hash, &["inserted".to_string()]).unwrap();
        let new_lines: Vec<&str> = result.lines().collect();
        assert_eq!(new_lines.len(), 4);
        assert_eq!(new_lines[1], "inserted");
    }
}
