//! Source File Management
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Manages source file metadata and provides utilities for mapping byte offsets
//! to line/column positions. This enables accurate diagnostic reporting with
//! human-readable locations.
//!
//! COMPILER PHASE: Driver (infrastructure)
//! INPUT: File path and source text
//! OUTPUT: SourceFile with line indexing for diagnostics
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Precomputed line starts: Build index once during construction.
//!   WHY: Repeated offset-to-line-col conversions during error reporting would be
//!   O(n) each time; precomputation makes lookups O(log n) via binary search.
//! - Support inline sources: `SourceFile::inline` for REPL or test cases.
//!   WHY: Not all Faber code comes from disk files (string compilation, LSP).

use std::path::PathBuf;

/// A source file with precomputed line indexing.
///
/// WHY: Line/column lookups are frequent during error reporting; precomputing
/// line starts amortizes the O(n) indexing cost across all diagnostics.
///
/// INVARIANTS:
/// ----------
/// INV-1: line_starts[0] is always 0 (first line starts at byte 0).
/// INV-2: line_starts is sorted in ascending order (monotonic byte offsets).
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

        Self { path, name, content, line_starts }
    }

    /// Create from just content (for inline compilation)
    pub fn inline(name: impl Into<String>, content: String) -> Self {
        let name = name.into();
        let line_starts = compute_line_starts(&content);

        Self { path: PathBuf::new(), name, content, line_starts }
    }

    /// Convert byte offset to 1-based line and column numbers.
    ///
    /// WHY: Diagnostics display line:col in human-readable form (editors use 1-based).
    ///
    /// PERF: Binary search in precomputed line_starts is O(log n).
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

        Some(self.content[start..end].trim_end_matches('\n'))
    }
}

/// Compute byte offsets of each line start.
///
/// WHY: Enables O(log n) offset-to-line conversion via binary search.
///
/// NOTE: Handles both \n and \r\n line endings (only \n is recorded).
fn compute_line_starts(content: &str) -> Vec<u32> {
    let mut starts = vec![0];

    for (i, c) in content.char_indices() {
        if c == '\n' {
            starts.push((i + 1) as u32);
        }
    }

    starts
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
