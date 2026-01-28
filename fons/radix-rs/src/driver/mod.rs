//! Compilation driver
//!
//! Orchestrates the compilation pipeline from source to output.

mod session;
mod source;

pub use session::{Config, Session};
pub use source::SourceFile;

use crate::codegen::{self, Target};
use crate::diagnostics::Diagnostic;
use crate::lexer;
use crate::parser;
use crate::semantic::{self, PassConfig};
use crate::syntax::*;
use crate::{CompileResult, Output};

/// Run the full compilation pipeline
pub fn compile(session: &Session, name: &str, source: &str) -> CompileResult {
    let mut diagnostics = Vec::new();

    // Phase 1: Lexing
    let lex_result = lexer::lex(source);
    if !lex_result.success() {
        for err in &lex_result.errors {
            diagnostics.push(Diagnostic::from_lex_error(name, source, err));
        }
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    // Phase 2: Parsing
    let parse_result = parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            diagnostics.push(Diagnostic::from_parse_error(name, source, err));
        }
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    let program = parse_result.program.unwrap();

    if session.config.target == Target::Rust {
        diagnostics.extend(collect_rust_noop_warnings(&program, name));
    }

    // Phase 3: Semantic analysis
    let pass_config = PassConfig::for_target(session.config.target);
    let semantic_result = semantic::analyze(&program, &pass_config);

    for err in &semantic_result.errors {
        diagnostics.push(Diagnostic::from_semantic_error(name, source, err));
    }

    if !semantic_result.success() {
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    let hir = semantic_result.hir.unwrap();

    // Phase 4: Code generation
    let crate_name = session.config.crate_name.as_deref().unwrap_or("output");
    match codegen::generate(
        session.config.target,
        &hir,
        &semantic_result.types,
        crate_name,
    ) {
        Ok(output) => CompileResult {
            output: Some(output),
            diagnostics,
        },
        Err(err) => {
            diagnostics.push(Diagnostic::codegen_error(&err.message));
            CompileResult {
                output: None,
                diagnostics,
            }
        }
    }
}

fn collect_rust_noop_warnings(program: &Program, file: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for stmt in &program.stmts {
        scan_stmt_for_rust_warnings(stmt, file, &mut diagnostics);
    }
    diagnostics
}

fn scan_stmt_for_rust_warnings(stmt: &Stmt, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match &stmt.kind {
        StmtKind::Block(block) => scan_block_for_rust_warnings(block, file, diagnostics),
        StmtKind::Func(func) => {
            if let Some(body) = &func.body {
                scan_block_for_rust_warnings(body, file, diagnostics);
            }
        }
        StmtKind::Class(class) => {
            for member in &class.members {
                if let ClassMemberKind::Method(method) = &member.kind {
                    if let Some(body) = &method.body {
                        scan_block_for_rust_warnings(body, file, diagnostics);
                    }
                }
            }
        }
        StmtKind::Probandum(test) => scan_test_for_rust_warnings(test, file, diagnostics),
        StmtKind::Proba(test) => scan_block_for_rust_warnings(&test.body, file, diagnostics),
        StmtKind::Si(if_stmt) => {
            scan_if_body_for_rust_warnings(&if_stmt.then, file, diagnostics);
            if let Some(catch) = &if_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
            if let Some(else_) = &if_stmt.else_ {
                scan_else_for_rust_warnings(else_, file, diagnostics);
            }
        }
        StmtKind::Dum(while_stmt) => {
            scan_if_body_for_rust_warnings(&while_stmt.body, file, diagnostics);
            if let Some(catch) = &while_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Itera(iter_stmt) => {
            scan_if_body_for_rust_warnings(&iter_stmt.body, file, diagnostics);
            if let Some(catch) = &iter_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Elige(switch_stmt) => {
            for case in &switch_stmt.cases {
                scan_if_body_for_rust_warnings(&case.body, file, diagnostics);
            }
            if let Some(default) = &switch_stmt.default {
                scan_if_body_for_rust_warnings(&default.body, file, diagnostics);
            }
            if let Some(catch) = &switch_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Discerne(match_stmt) => {
            for arm in &match_stmt.arms {
                scan_if_body_for_rust_warnings(&arm.body, file, diagnostics);
            }
            if let Some(default) = &match_stmt.default {
                scan_if_body_for_rust_warnings(&default.body, file, diagnostics);
            }
        }
        StmtKind::Custodi(guard_stmt) => {
            for clause in &guard_stmt.clauses {
                scan_if_body_for_rust_warnings(&clause.body, file, diagnostics);
            }
        }
        StmtKind::Fac(fac_stmt) => {
            scan_block_for_rust_warnings(&fac_stmt.body, file, diagnostics);
            if let Some(catch) = &fac_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Tempta(try_stmt) => {
            scan_block_for_rust_warnings(&try_stmt.body, file, diagnostics);
            if let Some(catch) = &try_stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
            if let Some(finally) = &try_stmt.finally {
                scan_block_for_rust_warnings(finally, file, diagnostics);
            }
        }
        StmtKind::Incipit(entry) => {
            scan_if_body_for_rust_warnings(&entry.body, file, diagnostics);
        }
        StmtKind::Cura(resource) => {
            if matches!(resource.kind, Some(CuraKind::Arena)) {
                let spec =
                    crate::diagnostics::semantic_spec(crate::semantic::SemanticErrorKind::Warning(
                        crate::semantic::WarningKind::TargetNoop,
                    ));
                let mut diag = Diagnostic::warning("cura arena has no effect for Rust targets")
                    .with_code(spec.code)
                    .with_file(file)
                    .with_span(stmt.span);
                if let Some(help) = spec.help {
                    diag = diag.with_help(help);
                }
                diagnostics.push(diag);
            }
            scan_block_for_rust_warnings(&resource.body, file, diagnostics);
            if let Some(catch) = &resource.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Ad(endpoint) => {
            if let Some(body) = &endpoint.body {
                scan_block_for_rust_warnings(body, file, diagnostics);
            }
            if let Some(catch) = &endpoint.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
        }
        _ => {}
    }
}

fn scan_block_for_rust_warnings(block: &BlockStmt, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    for stmt in &block.stmts {
        scan_stmt_for_rust_warnings(stmt, file, diagnostics);
    }
}

fn scan_if_body_for_rust_warnings(body: &IfBody, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match body {
        IfBody::Block(block) => scan_block_for_rust_warnings(block, file, diagnostics),
        IfBody::Ergo(stmt) => scan_stmt_for_rust_warnings(stmt, file, diagnostics),
        IfBody::InlineReturn(_) => {}
    }
}

fn scan_else_for_rust_warnings(else_: &SecusClause, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match else_ {
        SecusClause::Sin(stmt) => {
            scan_if_body_for_rust_warnings(&stmt.then, file, diagnostics);
            if let Some(catch) = &stmt.catch {
                scan_block_for_rust_warnings(&catch.body, file, diagnostics);
            }
            if let Some(else_clause) = &stmt.else_ {
                scan_else_for_rust_warnings(else_clause, file, diagnostics);
            }
        }
        SecusClause::Block(block) => scan_block_for_rust_warnings(block, file, diagnostics),
        SecusClause::Stmt(stmt) => scan_stmt_for_rust_warnings(stmt, file, diagnostics),
        SecusClause::InlineReturn(_) => {}
    }
}

fn scan_test_for_rust_warnings(
    test: &ProbandumDecl,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for setup in &test.body.setup {
        scan_block_for_rust_warnings(&setup.body, file, diagnostics);
    }
    for case in &test.body.tests {
        scan_block_for_rust_warnings(&case.body, file, diagnostics);
    }
    for nested in &test.body.nested {
        scan_test_for_rust_warnings(nested, file, diagnostics);
    }
}
