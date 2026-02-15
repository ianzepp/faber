//! Diagnostic Rendering
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Renders diagnostics to human-readable output using the `ariadne` crate for
//! pretty-printed, color-coded error messages with source snippets and underlines.
//!
//! COMPILER PHASE: Diagnostics (infrastructure)
//! INPUT: Diagnostic structs and source file map
//! OUTPUT: Rendered error messages to stderr
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Leverage ariadne: Use a mature error rendering library rather than custom formatting.
//!   WHY: ariadne provides color, source snippets, and multi-line spans out of the box.
//! - Color-coded severity: Errors are red, warnings are yellow, info is blue.
//!   WHY: Visual distinction helps users prioritize fixing errors over warnings.
//! - Source context: Show the problematic line with an underline highlight.
//!   WHY: Users need to see the error in context without opening the file.

use super::{Diagnostic, Severity};
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Render diagnostics to stderr using ariadne.
///
/// WHY: Batch rendering allows grouped output and deduplication if needed.
pub fn render_diagnostics(diagnostics: &[Diagnostic], sources: &[(String, String)]) {
    for diag in diagnostics {
        render_one(diag, sources);
    }
}

fn render_one(diag: &Diagnostic, sources: &[(String, String)]) {
    let kind = match diag.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
        Severity::Info => ReportKind::Advice,
    };

    let message = if let Some(code) = diag.code {
        format!("[{}] {}", code, diag.message)
    } else {
        diag.message.clone()
    };

    // Find the source for this file
    let source = sources
        .iter()
        .find(|(name, _)| name == &diag.file)
        .map(|(_, content)| content.as_str())
        .unwrap_or("");

    let mut builder =
        Report::build(kind, &diag.file, diag.span.map(|s| s.start as usize).unwrap_or(0)).with_message(message);

    if let Some(span) = diag.span {
        let color = match diag.severity {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Info => Color::Blue,
        };

        builder = builder.with_label(
            Label::new((&diag.file, span.start as usize..span.end as usize))
                .with_color(color)
                .with_message(&diag.message),
        );
    }

    if let Some(help) = &diag.help {
        builder = builder.with_help(help);
    }

    let _ = builder.finish().eprint((&diag.file, Source::from(source)));
}

/// Render diagnostic as plain text without colors.
///
/// WHY: Useful for tests, CI logs, and environments without TTY color support.
#[allow(dead_code)]
pub fn render_plain(diag: &Diagnostic) -> String {
    let severity = match diag.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };

    let location = if let Some(span) = diag.span {
        format!("{}:{}", diag.file, span.start)
    } else {
        diag.file.clone()
    };

    let mut output = if let Some(code) = diag.code {
        format!("{}[{}]: {}: {}", severity, code, location, diag.message)
    } else {
        format!("{}: {}: {}", severity, location, diag.message)
    };

    if let Some(line) = &diag.source_line {
        output.push_str(&format!("\n  | {}", line));
    }

    if let Some(help) = &diag.help {
        output.push_str(&format!("\n  = help: {}", help));
    }

    output
}
