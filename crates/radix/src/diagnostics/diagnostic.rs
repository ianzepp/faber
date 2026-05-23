//! Canonical diagnostic data model.
//!
//! This module is the normalization boundary between compiler-internal error
//! types and reportable diagnostics. Lexer, parser, and semantic analysis keep
//! their own rich error enums, but anything leaving those phases is converted
//! into [`Diagnostic`] so renderers, CLIs, and tests can reason about one
//! severity/code/span/help contract.
//!
//! The model intentionally stores display-ready strings and optional source
//! context instead of borrowing phase errors. Diagnostics may outlive a phase
//! value, be accumulated across a session, and still render even when source
//! context is unavailable.
//!
//! INVARIANTS
//! ==========
//! - Catalog-derived diagnostics carry stable codes from `catalog`.
//! - Semantic warnings and errors share the same type; severity is the contract
//!   that decides whether compilation may proceed.
//! - Missing file, span, source line, or help information is valid for
//!   infrastructure failures and late backend errors.

use super::catalog;
use crate::lexer::{LexError, Span};
use crate::parser::ParseError;
use crate::semantic::SemanticError;
use std::fmt;
use std::path::Path;

/// User-visible severity and compilation policy for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Fatal diagnostic that should prevent successful compilation.
    Error,

    /// Non-fatal diagnostic for suspicious code or discouraged usage.
    Warning,

    /// Advisory note that explains context without indicating a fault.
    Info,
}

/// Compiler phase that produced or normalized a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticPhase {
    /// Filesystem or source-loading failure.
    Io,

    /// Lexer/tokenization failure.
    Lex,

    /// Parser/grammar failure.
    Parse,

    /// Name resolution failure.
    Resolve,

    /// AST-to-HIR lowering failure.
    Lower,

    /// Type checking failure.
    Typecheck,

    /// Coarse semantic analysis failure when no narrower pass is exposed.
    Analysis,

    /// MIR lowering, validation, or inspection failure.
    Mir,

    /// Target code generation failure.
    Codegen,

    /// CLI/tooling policy failure outside a compiler phase.
    Tool,
}

impl fmt::Display for DiagnosticPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            DiagnosticPhase::Io => "io",
            DiagnosticPhase::Lex => "lex",
            DiagnosticPhase::Parse => "parse",
            DiagnosticPhase::Resolve => "resolve",
            DiagnosticPhase::Lower => "lower",
            DiagnosticPhase::Typecheck => "typecheck",
            DiagnosticPhase::Analysis => "analysis",
            DiagnosticPhase::Mir => "mir",
            DiagnosticPhase::Codegen => "codegen",
            DiagnosticPhase::Tool => "tool",
        };
        f.write_str(name)
    }
}

/// Reportable compiler message after phase-specific errors are normalized.
///
/// This is the transport contract between compiler phases and presentation
/// layers. It intentionally keeps fields simple so callers can aggregate,
/// render, snapshot, or serialize diagnostics without retaining phase-specific
/// error values.
#[derive(Debug)]
pub struct Diagnostic {
    /// Compilation policy and visual priority for the message.
    pub severity: Severity,

    /// Compiler phase that owns the diagnostic.
    pub phase: DiagnosticPhase,

    /// Human-readable problem statement without the diagnostic code prefix.
    pub message: String,

    /// Stable catalog code used by docs, tests, and external tooling.
    pub code: Option<&'static str>,

    /// Source filename or path to display in reports.
    pub file: String,

    /// Byte span in `file`; absent when the error is not tied to source text.
    pub span: Option<Span>,

    /// Full source line containing `span`, captured for plain renderers.
    pub source_line: Option<String>,

    /// Actionable guidance shown after the primary message.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Start an error diagnostic and attach context with builder methods.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            phase: DiagnosticPhase::Tool,
            message: message.into(),
            code: None,
            file: String::new(),
            span: None,
            source_line: None,
            help: None,
        }
    }

    /// Start a warning diagnostic and attach context with builder methods.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            phase: DiagnosticPhase::Tool,
            message: message.into(),
            code: None,
            file: String::new(),
            span: None,
            source_line: None,
            help: None,
        }
    }

    /// Attach the compiler phase that owns this diagnostic.
    pub fn with_phase(mut self, phase: DiagnosticPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Attach a stable diagnostic code from the public catalog.
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    /// Override severity when command policy intentionally downgrades a fault.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Attach the display filename used by renderers and plain output.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    /// Attach the source span that should be highlighted.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attach the source line containing the primary span.
    pub fn with_source_line(mut self, line: impl Into<String>) -> Self {
        self.source_line = Some(line.into());
        self
    }

    /// Attach actionable user guidance.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Whether this diagnostic should make the compilation result fail.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Convert a filesystem read failure into a path-scoped diagnostic.
    pub fn io_error(path: &Path, err: std::io::Error) -> Self {
        Self::error(format!("cannot read '{}': {}", path.display(), err))
            .with_phase(DiagnosticPhase::Io)
            .with_file(path.display().to_string())
    }

    /// Normalize a lexer error using the diagnostic code catalog.
    ///
    /// Lexer errors already carry a concrete byte span, so this conversion also
    /// captures the containing source line for non-ariadne renderers.
    pub fn from_lex_error(file: &str, source: &str, err: &LexError) -> Self {
        let line = get_line_at_offset(source, err.span.start as usize);
        let spec = catalog::lex_spec(err.kind);
        Self::error(&err.message)
            .with_phase(DiagnosticPhase::Lex)
            .with_code(spec.code)
            .with_file(file)
            .with_span(err.span)
            .with_source_line(line)
            .with_help_opt(spec.help)
    }

    /// Normalize a parser error using the diagnostic code catalog.
    ///
    /// Parser recovery can produce multiple errors over the same token stream;
    /// this conversion keeps each one independently renderable with its own
    /// span and help contract.
    pub fn from_parse_error(file: &str, source: &str, err: &ParseError) -> Self {
        let line = get_line_at_offset(source, err.span.start as usize);
        let spec = catalog::parse_spec(err.kind);
        Self::error(&err.message)
            .with_phase(DiagnosticPhase::Parse)
            .with_code(spec.code)
            .with_file(file)
            .with_span(err.span)
            .with_source_line(line)
            .with_help_opt(spec.help)
    }

    /// Normalize a semantic error while preserving semantic warning policy.
    ///
    /// Semantic analysis is the first phase that can emit both fatal errors and
    /// warnings. Custom help carried by the semantic error wins over catalog
    /// help because it can include context discovered during analysis.
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

        Self {
            severity,
            phase: DiagnosticPhase::Analysis,
            message: err.message.clone(),
            code: Some(spec.code),
            file: file.to_owned(),
            span: Some(err.span),
            source_line: Some(line),
            help,
        }
    }

    /// Create a backend diagnostic when generation fails outside source spans.
    pub fn codegen_error(message: &str) -> Self {
        Self::error(format!("code generation failed: {}", message))
            .with_phase(DiagnosticPhase::Codegen)
            .with_code("CODEGEN001")
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

/// Extract the source line containing a byte offset.
///
/// WHY: Diagnostics need to show the problematic line for context.
fn get_line_at_offset(source: &str, offset: usize) -> String {
    let start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    source[start..end].to_owned()
}
