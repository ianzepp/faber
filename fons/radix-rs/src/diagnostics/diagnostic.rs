//! Diagnostic types

use super::catalog;
use crate::lexer::{LexError, Span};
use crate::parser::ParseError;
use crate::semantic::SemanticError;
use std::path::Path;

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A diagnostic message
#[derive(Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub code: Option<&'static str>,
    pub file: String,
    pub span: Option<Span>,
    pub source_line: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            code: None,
            file: String::new(),
            span: None,
            source_line: None,
            help: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            code: None,
            file: String::new(),
            span: None,
            source_line: None,
            help: None,
        }
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_source_line(mut self, line: impl Into<String>) -> Self {
        self.source_line = Some(line.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Create from IO error
    pub fn io_error(path: &Path, err: std::io::Error) -> Self {
        Self::error(format!("cannot read '{}': {}", path.display(), err))
            .with_file(path.display().to_string())
    }

    /// Create from lex error
    pub fn from_lex_error(file: &str, source: &str, err: &LexError) -> Self {
        let line = get_line_at_offset(source, err.span.start as usize);
        let spec = catalog::lex_spec(err.kind);
        Self::error(&err.message)
            .with_code(spec.code)
            .with_file(file)
            .with_span(err.span)
            .with_source_line(line)
            .with_help_opt(spec.help)
    }

    /// Create from parse error
    pub fn from_parse_error(file: &str, source: &str, err: &ParseError) -> Self {
        let line = get_line_at_offset(source, err.span.start as usize);
        let spec = catalog::parse_spec(err.kind);
        Self::error(&err.message)
            .with_code(spec.code)
            .with_file(file)
            .with_span(err.span)
            .with_source_line(line)
            .with_help_opt(spec.help)
    }

    /// Create from semantic error
    pub fn from_semantic_error(file: &str, source: &str, err: &SemanticError) -> Self {
        let line = get_line_at_offset(source, err.span.start as usize);
        let severity = if err.is_error() {
            Severity::Error
        } else {
            Severity::Warning
        };

        let spec = catalog::semantic_spec(err.kind);
        let help = if err.help.is_some() {
            err.help.clone()
        } else {
            spec.help.map(|h| h.to_owned())
        };

        let mut diag = Self {
            severity,
            message: err.message.clone(),
            code: Some(spec.code),
            file: file.to_owned(),
            span: Some(err.span),
            source_line: Some(line),
            help,
        };

        diag
    }

    /// Create codegen error
    pub fn codegen_error(message: &str) -> Self {
        Self::error(format!("code generation failed: {}", message)).with_code("CODEGEN001")
    }
}

impl Diagnostic {
    fn with_help_opt(mut self, help: Option<&'static str>) -> Self {
        if let Some(help) = help {
            self.help = Some(help.to_owned());
        }
        self
    }
}

fn get_line_at_offset(source: &str, offset: usize) -> String {
    let start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    source[start..end].to_owned()
}
