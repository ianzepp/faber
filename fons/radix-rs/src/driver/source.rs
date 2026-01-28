//! Source file management

use std::path::{Path, PathBuf};

/// A source file
#[derive(Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub name: String,
    pub content: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Create from path and content
    pub fn new(path: PathBuf, content: String) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unknown>".to_owned());

        let line_starts = compute_line_starts(&content);

        Self {
            path,
            name,
            content,
            line_starts,
        }
    }

    /// Create from just content (for inline compilation)
    pub fn inline(name: impl Into<String>, content: String) -> Self {
        let name = name.into();
        let line_starts = compute_line_starts(&content);

        Self {
            path: PathBuf::new(),
            name,
            content,
            line_starts,
        }
    }

    /// Convert byte offset to line and column
    pub fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);

        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let column = offset - line_start;

        (line as u32 + 1, column + 1)
    }

    /// Get the content of a specific line
    pub fn line_content(&self, line: u32) -> Option<&str> {
        let line_idx = line.saturating_sub(1) as usize;
        let start = *self.line_starts.get(line_idx)? as usize;
        let end = self
            .line_starts
            .get(line_idx + 1)
            .map(|&e| e as usize)
            .unwrap_or(self.content.len());

        Some(&self.content[start..end].trim_end_matches('\n'))
    }
}

fn compute_line_starts(content: &str) -> Vec<u32> {
    let mut starts = vec![0];

    for (i, c) in content.char_indices() {
        if c == '\n' {
            starts.push((i + 1) as u32);
        }
    }

    starts
}
