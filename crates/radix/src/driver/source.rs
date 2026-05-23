//! Source text identity and byte-position indexing.
//!
//! Diagnostics are produced by compiler phases in byte offsets, but humans and
//! editors need stable file names, line numbers, and displayable line excerpts.
//! [`SourceFile`] is the driver-side adapter between those two views: it stores
//! source text with its origin and precomputes line starts once so every later
//! diagnostic lookup can be cheap and deterministic.
//!
//! INVARIANTS
//! ==========
//! - `line_starts[0]` is always `0`, even for empty input.
//! - Line starts are byte offsets, not character indexes; spans throughout the
//!   compiler use byte positions.
//! - Displayed line and column numbers are one-based.
//! - Inline sources keep an empty path but still carry a user-facing name for
//!   diagnostics, REPLs, and tests.
//!
//! PERFORMANCE
//! ===========
//! Position conversion uses a binary search over precomputed line starts. That
//! keeps repeated diagnostics proportional to `log(lines)` instead of rescanning
//! the full source for every span.

use std::path::PathBuf;

/// Source text plus the lookup table needed for diagnostic rendering.
#[derive(Debug)]
pub struct SourceFile {
    /// Filesystem origin for disk-backed sources; empty for inline sources.
    pub path: PathBuf,

    /// User-facing name used in diagnostics.
    pub name: String,

    /// Complete UTF-8 source text.
    pub content: String,

    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Build a disk-backed source file from its path and contents.
    ///
    /// The display name is derived from the final path component so diagnostics
    /// stay readable even when callers pass absolute paths.
    pub fn new(path: PathBuf, content: String) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unknown>".to_owned());

        let line_starts = compute_line_starts(&content);

        Self { path, name, content, line_starts }
    }

    /// Build an inline source file with no filesystem path.
    ///
    /// Inline inputs are used by string compilation, tests, and tool surfaces
    /// where diagnostics still need a stable source name.
    pub fn inline(name: impl Into<String>, content: String) -> Self {
        let name = name.into();
        let line_starts = compute_line_starts(&content);

        Self { path: PathBuf::new(), name, content, line_starts }
    }

    /// Convert a byte offset to one-based line and column numbers.
    ///
    /// Offsets beyond the final recorded line start are anchored to the last
    /// known line. The column is byte-based to match compiler spans.
    pub fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);

        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let column = offset - line_start;

        (line as u32 + 1, column + 1)
    }

    /// Return one display line by one-based line number.
    ///
    /// The returned slice trims the trailing line feed so diagnostic renderers
    /// can place carets without carrying source line endings into output.
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

/// Compute byte offsets of each line start for binary-search lookup.
///
/// EDGE: `\r\n` input records the byte after `\n`; any preceding `\r` remains
/// part of the line slice because display trimming only removes the line feed.
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
