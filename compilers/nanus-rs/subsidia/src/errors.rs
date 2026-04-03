use crate::Locus;
use std::fmt;

/// Compile error with source position.
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub locus: Locus,
    pub filename: String,
}

impl CompileError {
    pub fn new(message: impl Into<String>, locus: Locus, filename: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            locus,
            filename: filename.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.filename, self.locus.linea, self.locus.columna, self.message
        )
    }
}

impl std::error::Error for CompileError {}

/// Format a human-friendly error message with source context.
pub fn format_error(err: &CompileError, source: &str) -> String {
    let lines: Vec<&str> = source.split('\n').collect();
    let line_idx = (err.locus.linea - 1) as usize;
    let src_line = lines.get(line_idx).unwrap_or(&"");

    let col = (err.locus.columna - 1).max(0) as usize;
    let pointer = " ".repeat(col) + "^";

    format!(
        "{}:{}:{}: error: {}\n\n  {}\n  {}",
        err.filename, err.locus.linea, err.locus.columna, err.message, src_line, pointer
    )
}
