//! Semantic `check` command and normalized diagnostic collection.

use super::super::cli::{CheckCommand, DiagnosticMode};
use super::package::should_treat_as_package_from_input;
use super::source::{format_location, read_source, source_file_from_input};

pub fn cmd_check(command: CheckCommand) {
    if command.package || should_treat_as_package_from_input(&command.input) {
        eprintln!("error: package checking is owned by the `faber` tool; rerun with `faber check --package`");
        std::process::exit(1);
    }

    let (name, source) = read_source(&command.input);
    let source_file = source_file_from_input(name, source);

    if command.diagnostic_mode == DiagnosticMode::Diagnostics {
        let result = check_diagnostics_for_source(&source_file.name, &source_file.content, command.permissive);
        if !result.diagnostics.is_empty() {
            eprintln!("{}", crate::diagnostics::render_expanded_diagnostics(&result.diagnostics));
        }
        if result.success {
            eprintln!("ok: {}", source_file.name.as_str());
        } else {
            std::process::exit(1);
        }
        return;
    }

    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        for err in &lex_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    let mut cli_fatal_errors = 0usize;
    for err in &cli_analysis.errors {
        eprintln!("error: {}: {}", format_location(&source_file, err.span.start), err.message);
        cli_fatal_errors += 1;
    }
    if cli_fatal_errors > 0 {
        std::process::exit(1);
    }

    let pass_config = crate::semantic::PassConfig::for_target(crate::codegen::Target::Rust);
    let semantic_result =
        crate::semantic::analyze_with_cli(&program, &pass_config, &interner, cli_analysis.program.as_ref());

    let mut fatal_errors = 0usize;
    let mut downgraded = 0usize;
    for err in &semantic_result.errors {
        let downgraded_error = command.permissive && err.kind.is_permissive_check_downgrade();
        let prefix = if err.is_error() && !downgraded_error {
            "error"
        } else {
            "warning"
        };
        eprintln!("{}: {}: {}", prefix, format_location(&source_file, err.span.start), err.message);
        if err.is_error() {
            if downgraded_error {
                downgraded += 1;
            } else {
                fatal_errors += 1;
            }
        }
    }

    if command.permissive && downgraded > 0 {
        eprintln!(
            "warning:{}: downgraded {} unresolved/import-driven semantic error(s) in permissive mode",
            source_file.name.as_str(),
            downgraded
        );
    }

    if semantic_result.success() || (command.permissive && fatal_errors == 0) {
        eprintln!("ok: {}", source_file.name.as_str());
    } else {
        std::process::exit(1);
    }
}

/// Result of running check through normalized diagnostics.
#[derive(Debug)]
pub struct CheckDiagnosticsResult {
    /// Normalized diagnostics collected before check completion.
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,

    /// Whether check policy accepts the input after diagnostics are considered.
    pub success: bool,
}

/// Run single-file check and collect normalized diagnostics without exiting.
pub fn check_diagnostics_for_source(name: &str, source: &str, permissive: bool) -> CheckDiagnosticsResult {
    let mut diagnostics = Vec::new();

    let lex_result = crate::lexer::lex(source);
    if !lex_result.success() {
        diagnostics.extend(
            lex_result
                .errors
                .iter()
                .map(|err| crate::diagnostics::Diagnostic::from_lex_error(name, source, err)),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        diagnostics.extend(
            parse_result
                .errors
                .iter()
                .map(|err| crate::diagnostics::Diagnostic::from_parse_error(name, source, err)),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        diagnostics.push(
            crate::diagnostics::Diagnostic::error("successful parse result missing program")
                .with_phase(crate::diagnostics::DiagnosticPhase::Parse)
                .with_file(name.to_owned()),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    diagnostics.extend(
        cli_analysis
            .errors
            .iter()
            .map(|err| crate::diagnostics::Diagnostic::from_semantic_error(name, source, err)),
    );
    if diagnostics
        .iter()
        .any(crate::diagnostics::Diagnostic::is_error)
    {
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let pass_config = crate::semantic::PassConfig::for_target(crate::codegen::Target::Rust);
    let semantic_result =
        crate::semantic::analyze_with_cli(&program, &pass_config, &interner, cli_analysis.program.as_ref());

    let mut fatal_errors = 0usize;
    let mut downgraded = 0usize;
    for err in &semantic_result.errors {
        let downgraded_error = permissive && err.kind.is_permissive_check_downgrade();
        let mut diagnostic = crate::diagnostics::Diagnostic::from_semantic_error(name, source, err);
        if downgraded_error {
            diagnostic = diagnostic.with_severity(crate::diagnostics::Severity::Warning);
            downgraded += 1;
        } else if err.is_error() {
            fatal_errors += 1;
        }
        diagnostics.push(diagnostic);
    }

    if permissive && downgraded > 0 {
        diagnostics.push(
            crate::diagnostics::Diagnostic::warning(format!(
                "downgraded {} unresolved/import-driven semantic error(s) in permissive mode",
                downgraded
            ))
            .with_phase(crate::diagnostics::DiagnosticPhase::Tool)
            .with_file(name.to_owned()),
        );
    }

    CheckDiagnosticsResult { diagnostics, success: semantic_result.success() || (permissive && fatal_errors == 0) }
}
