//! Diagnostic rendering
//!
//! Uses ariadne for pretty error output.

use super::{Diagnostic, Severity};
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Render diagnostics to stderr
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

    // Find the source for this file
    let source = sources.iter()
        .find(|(name, _)| name == &diag.file)
        .map(|(_, content)| content.as_str())
        .unwrap_or("");

    let mut builder = Report::build(kind, &diag.file, diag.span.map(|s| s.start as usize).unwrap_or(0))
        .with_message(&diag.message);

    if let Some(span) = diag.span {
        let color = match diag.severity {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Info => Color::Blue,
        };

        builder = builder.with_label(
            Label::new((&diag.file, span.start as usize..span.end as usize))
                .with_color(color)
                .with_message(&diag.message)
        );
    }

    if let Some(help) = &diag.help {
        builder = builder.with_help(help);
    }

    let _ = builder.finish().eprint((&diag.file, Source::from(source)));
}

/// Simple text rendering (no colors)
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

    let mut output = format!("{}: {}: {}", severity, location, diag.message);

    if let Some(line) = &diag.source_line {
        output.push_str(&format!("\n  | {}", line));
    }

    if let Some(help) = &diag.help {
        output.push_str(&format!("\n  = help: {}", help));
    }

    output
}
