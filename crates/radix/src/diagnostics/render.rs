//! Terminal rendering boundary for normalized diagnostics.
//!
//! Rendering is intentionally downstream of diagnostic construction. By the
//! time a value reaches this module, it already has a severity, optional code,
//! display file, source span, and help text; this file decides only how that
//! contract appears in terminal and plain-text output.
//!
//! The pretty renderer delegates span layout and color handling to `ariadne`.
//! The plain renderer stays local because tests, CI logs, and non-TTY consumers
//! need deterministic text without depending on terminal capabilities.
//!
//! INVARIANTS
//! ==========
//! - Rendering must not panic when source text is missing; callers may report
//!   diagnostics for generated or filesystem-only failures.
//! - Codes are prefixed to the primary message before presentation.
//! - Severity controls both ariadne report kind and label color.

use super::{Diagnostic, Severity};
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Render a batch of normalized diagnostics to stderr with source snippets.
///
/// `sources` is a simple `(display_name, source_text)` map because diagnostics
/// may be produced across package inputs. Missing entries are tolerated and
/// render as span-less context rather than failing the reporting path.
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

    // Missing source is valid for backend and filesystem diagnostics.
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

/// Render one diagnostic as deterministic plain text without terminal colors.
///
/// This mirrors the same semantic pieces as the ariadne renderer while keeping
/// output stable for snapshots and environments that cannot render rich spans.
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
