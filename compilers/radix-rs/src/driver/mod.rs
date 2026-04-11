//! Compilation driver - orchestrates the multi-phase pipeline
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! The driver module coordinates the execution of all compiler phases in sequence:
//! lexing → parsing → semantic analysis → codegen. It collects diagnostics from
//! each phase and halts on errors before attempting dependent phases.
//!
//! COMPILER PHASE: Driver (pipeline orchestration)
//! INPUT: Source code string, session configuration
//! OUTPUT: CompileResult with optional target code and diagnostics
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Fail fast: Stop after each failed phase to avoid cascading errors. For
//!   example, don't attempt parsing if lexing failed, as token stream is invalid.
//!
//! - Diagnostic collection: Gather all diagnostics from every phase, even when
//!   compilation succeeds (to report warnings).
//!
//! - Target-aware warnings: Some constructs are no-ops in certain targets
//!   (e.g., `cura arena` in Rust). The driver scans for these after parsing
//!   and emits warnings before semantic analysis.
//!
//! PIPELINE PHASES
//! ===============
//! 1. Lexing: Tokenize source
//! 2. Parsing: Build AST
//! 3. Target warnings: Scan for target-specific no-ops
//! 4. Semantic: Name resolution, type checking, borrow analysis
//! 5. Codegen: Emit target source code

mod project;
mod session;
mod source;

pub use project::compile_package;
pub use session::{Config, Session};
pub use source::SourceFile;

use crate::codegen::{self, Target};
use crate::diagnostics::Diagnostic;
use crate::hir::HirProgram;
use crate::lexer;
use crate::lexer::Interner;
use crate::parser;
use crate::semantic::{self, PassConfig};
use crate::syntax::*;
use crate::CompileResult;
use std::collections::HashSet;

// =============================================================================
// CORE
// =============================================================================
//
// Main compilation pipeline entry point.

/// Run the full compilation pipeline.
///
/// WHY: Single entry point for all compilation. Each phase is run sequentially,
/// with early exit on errors to avoid wasting work on invalid input.
///
/// PHASES:
/// - Lexing: Tokenize source
/// - Parsing: Build AST
/// - Target warnings: Scan for constructs that are no-ops in the selected target
/// - Semantic: Analyze types and borrows
/// - Codegen: Emit target code
pub fn compile(session: &Session, name: &str, source: &str) -> CompileResult {
    let mut analysis = match analyze_source(session, name, source) {
        Ok(analysis) => analysis,
        Err(diagnostics) => return CompileResult { output: None, diagnostics },
    };

    // -------------------------------------------------------------------------
    // PHASE 4: CODE GENERATION
    // Emit target-specific source code
    // -------------------------------------------------------------------------
    match codegen::generate(session.config.target, &analysis.hir, &analysis.types, &analysis.interner) {
        Ok(output) => CompileResult { output: Some(output), diagnostics: analysis.diagnostics },
        Err(err) => {
            analysis
                .diagnostics
                .push(Diagnostic::codegen_error(&err.message));
            CompileResult { output: None, diagnostics: analysis.diagnostics }
        }
    }
}

pub(crate) struct AnalyzedUnit {
    pub interner: Interner,
    pub types: semantic::TypeTable,
    pub hir: HirProgram,
    pub diagnostics: Vec<Diagnostic>,
}

pub(crate) fn analyze_source(session: &Session, name: &str, source: &str) -> Result<AnalyzedUnit, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    let lex_result = lexer::lex(source);
    if !lex_result.success() {
        for err in &lex_result.errors {
            diagnostics.push(Diagnostic::from_lex_error(name, source, err));
        }
        return Err(diagnostics);
    }

    let parse_result = parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            diagnostics.push(Diagnostic::from_parse_error(name, source, err));
        }
        return Err(diagnostics);
    }

    let parser::ParseResult { program, interner, .. } = parse_result;
    let program = program.expect("successful parse result must contain a program");

    match session.config.target {
        Target::Go => diagnostics.extend(collect_go_unsupported_errors(&program, name, &interner)),
        Target::Rust => {
            diagnostics.extend(collect_rust_unsupported_errors(&program, name));
            diagnostics.extend(collect_rust_noop_warnings(&program, name));
        }
        _ => {}
    }

    if diagnostics.iter().any(Diagnostic::is_error) {
        return Err(diagnostics);
    }

    let pass_config = PassConfig::for_target(session.config.target);
    let semantic_result = semantic::analyze(&program, &pass_config, &interner);

    for err in &semantic_result.errors {
        diagnostics.push(Diagnostic::from_semantic_error(name, source, err));
    }

    if !semantic_result.success() {
        return Err(diagnostics);
    }

    Ok(AnalyzedUnit {
        interner,
        types: semantic_result.types,
        hir: semantic_result
            .hir
            .expect("successful semantic result must contain lowered HIR"),
        diagnostics,
    })
}

// =============================================================================
// HELPERS
// =============================================================================
//
// Target-specific validation and warning detection.

fn collect_rust_unsupported_errors(program: &Program, file: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for stmt in &program.stmts {
        scan_stmt_for_rust_unsupported_errors(stmt, file, &mut diagnostics);
    }
    diagnostics
}

fn collect_go_unsupported_errors(program: &Program, file: &str, interner: &Interner) -> Vec<Diagnostic> {
    let dynamic_externa = collect_go_dynamic_externa(program, interner);
    let mut diagnostics = Vec::new();

    for stmt in &program.stmts {
        scan_stmt_for_go_unsupported_errors(stmt, file, &dynamic_externa, &mut diagnostics);
    }

    diagnostics
}

#[derive(Default)]
struct GoDynamicExterna {
    bindings: HashSet<crate::lexer::Symbol>,
    functions: HashSet<crate::lexer::Symbol>,
}

fn collect_go_dynamic_externa(program: &Program, interner: &Interner) -> GoDynamicExterna {
    let mut dynamic = GoDynamicExterna::default();

    for stmt in &program.stmts {
        match &stmt.kind {
            StmtKind::Var(decl) if has_externa_annotation(&stmt.annotations, &[], interner) => {
                if decl.ty.as_ref().is_some_and(|ty| is_named_type(ty, interner, "ignotum")) {
                    if let BindingPattern::Ident(ident) = &decl.binding {
                        dynamic.bindings.insert(ident.name);
                    }
                }
            }
            StmtKind::Func(func) if has_externa_annotation(&stmt.annotations, &func.annotations, interner) => {
                if func.ret.as_ref().is_some_and(|ty| is_named_type(ty, interner, "ignotum")) {
                    dynamic.functions.insert(func.name.name);
                }
            }
            _ => {}
        }
    }

    dynamic
}

fn has_externa_annotation(
    stmt_annotations: &[Annotation],
    decl_annotations: &[Annotation],
    interner: &Interner,
) -> bool {
    stmt_annotations
        .iter()
        .chain(decl_annotations.iter())
        .any(|annotation| match &annotation.kind {
            AnnotationKind::Externa => true,
            AnnotationKind::Statement(stmt) => interner.resolve(stmt.name.name) == "externa",
            _ => false,
        })
}

fn is_named_type(ty: &TypeExpr, interner: &Interner, expected: &str) -> bool {
    match &ty.kind {
        TypeExprKind::Named(name, params) => params.is_empty() && interner.resolve(name.name) == expected,
        _ => false,
    }
}

fn scan_stmt_for_go_unsupported_errors(
    stmt: &Stmt,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &stmt.kind {
        StmtKind::Block(block) => scan_block_for_go_unsupported_errors(block, file, dynamic_externa, diagnostics),
        StmtKind::Func(func) => {
            if let Some(body) = &func.body {
                scan_block_for_go_unsupported_errors(body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Class(class) => {
            for member in &class.members {
                if let ClassMemberKind::Method(method) = &member.kind {
                    if let Some(body) = &method.body {
                        scan_block_for_go_unsupported_errors(body, file, dynamic_externa, diagnostics);
                    }
                }
            }
        }
        StmtKind::Probandum(test) => {
            for setup in &test.body.setup {
                scan_block_for_go_unsupported_errors(&setup.body, file, dynamic_externa, diagnostics);
            }
            for case in &test.body.tests {
                scan_block_for_go_unsupported_errors(&case.body, file, dynamic_externa, diagnostics);
            }
            for nested in &test.body.nested {
                scan_probandum_for_go_unsupported_errors(nested, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Proba(test) => {
            scan_block_for_go_unsupported_errors(&test.body, file, dynamic_externa, diagnostics);
        }
        StmtKind::Si(if_stmt) => {
            scan_expr_for_go_unsupported_errors(&if_stmt.cond, file, dynamic_externa, diagnostics);
            scan_if_body_for_go_unsupported_errors(&if_stmt.then, file, dynamic_externa, diagnostics);
            if let Some(catch) = &if_stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
            if let Some(else_) = &if_stmt.else_ {
                scan_else_for_go_unsupported_errors(else_, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Dum(while_stmt) => {
            scan_expr_for_go_unsupported_errors(&while_stmt.cond, file, dynamic_externa, diagnostics);
            scan_if_body_for_go_unsupported_errors(&while_stmt.body, file, dynamic_externa, diagnostics);
            if let Some(catch) = &while_stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Itera(iter_stmt) => {
            scan_expr_for_go_unsupported_errors(&iter_stmt.iterable, file, dynamic_externa, diagnostics);
            scan_if_body_for_go_unsupported_errors(&iter_stmt.body, file, dynamic_externa, diagnostics);
            if let Some(catch) = &iter_stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Elige(switch_stmt) => {
            scan_expr_for_go_unsupported_errors(&switch_stmt.expr, file, dynamic_externa, diagnostics);
            for case in &switch_stmt.cases {
                scan_expr_for_go_unsupported_errors(&case.value, file, dynamic_externa, diagnostics);
                scan_if_body_for_go_unsupported_errors(&case.body, file, dynamic_externa, diagnostics);
            }
            if let Some(default) = &switch_stmt.default {
                scan_if_body_for_go_unsupported_errors(&default.body, file, dynamic_externa, diagnostics);
            }
            if let Some(catch) = &switch_stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Discerne(match_stmt) => {
            for subject in &match_stmt.subjects {
                scan_expr_for_go_unsupported_errors(subject, file, dynamic_externa, diagnostics);
            }
            for arm in &match_stmt.arms {
                scan_if_body_for_go_unsupported_errors(&arm.body, file, dynamic_externa, diagnostics);
            }
            if let Some(default) = &match_stmt.default {
                scan_if_body_for_go_unsupported_errors(&default.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Custodi(guard_stmt) => {
            for clause in &guard_stmt.clauses {
                scan_expr_for_go_unsupported_errors(&clause.cond, file, dynamic_externa, diagnostics);
                scan_if_body_for_go_unsupported_errors(&clause.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Fac(fac_stmt) => {
            scan_block_for_go_unsupported_errors(&fac_stmt.body, file, dynamic_externa, diagnostics);
            if let Some(catch) = &fac_stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
            if let Some(cond) = &fac_stmt.while_ {
                scan_expr_for_go_unsupported_errors(cond, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Redde(ret) => {
            if let Some(value) = &ret.value {
                scan_expr_for_go_unsupported_errors(value, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Iace(stmt) => {
            scan_expr_for_go_unsupported_errors(&stmt.value, file, dynamic_externa, diagnostics);
        }
        StmtKind::Mori(stmt) => {
            scan_expr_for_go_unsupported_errors(&stmt.value, file, dynamic_externa, diagnostics);
        }
        StmtKind::Tempta(stmt) => {
            scan_block_for_go_unsupported_errors(&stmt.body, file, dynamic_externa, diagnostics);
            if let Some(catch) = &stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
            if let Some(finally) = &stmt.finally {
                scan_block_for_go_unsupported_errors(finally, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Adfirma(stmt) => {
            scan_expr_for_go_unsupported_errors(&stmt.cond, file, dynamic_externa, diagnostics);
            if let Some(message) = &stmt.message {
                scan_expr_for_go_unsupported_errors(message, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Scribe(stmt) => {
            for arg in &stmt.args {
                scan_expr_for_go_unsupported_errors(arg, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Incipit(entry) => {
            if let Some(exitus) = &entry.exitus {
                scan_expr_for_go_unsupported_errors(exitus, file, dynamic_externa, diagnostics);
            }
            scan_if_body_for_go_unsupported_errors(&entry.body, file, dynamic_externa, diagnostics);
        }
        StmtKind::Cura(resource) => {
            if let Some(init) = &resource.init {
                scan_expr_for_go_unsupported_errors(init, file, dynamic_externa, diagnostics);
            }
            scan_block_for_go_unsupported_errors(&resource.body, file, dynamic_externa, diagnostics);
            if let Some(catch) = &resource.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Ad(endpoint) => {
            diagnostics.push(go_target_policy_diagnostic(
                file,
                stmt.span,
                "ad is not supported for Go targets",
                "mark 'ad' unsupported for Go, or route the effect through an explicit Go runtime function with a concrete return type",
            ));
            for arg in &endpoint.args {
                scan_expr_for_go_unsupported_errors(&arg.value, file, dynamic_externa, diagnostics);
            }
            if let Some(body) = &endpoint.body {
                scan_block_for_go_unsupported_errors(body, file, dynamic_externa, diagnostics);
            }
            if let Some(catch) = &endpoint.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Expr(expr) => scan_expr_for_go_unsupported_errors(&expr.expr, file, dynamic_externa, diagnostics),
        StmtKind::Var(decl) => {
            if let Some(init) = &decl.init {
                scan_expr_for_go_unsupported_errors(init, file, dynamic_externa, diagnostics);
            }
        }
        StmtKind::Ex(stmt) => scan_expr_for_go_unsupported_errors(&stmt.source, file, dynamic_externa, diagnostics),
        StmtKind::Import(_)
        | StmtKind::TypeAlias(_)
        | StmtKind::Enum(_)
        | StmtKind::Union(_)
        | StmtKind::Interface(_)
        | StmtKind::Rumpe(_)
        | StmtKind::Perge(_) => {}
    }
}

fn scan_probandum_for_go_unsupported_errors(
    test: &ProbandumDecl,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for setup in &test.body.setup {
        scan_block_for_go_unsupported_errors(&setup.body, file, dynamic_externa, diagnostics);
    }
    for case in &test.body.tests {
        scan_block_for_go_unsupported_errors(&case.body, file, dynamic_externa, diagnostics);
    }
    for nested in &test.body.nested {
        scan_probandum_for_go_unsupported_errors(nested, file, dynamic_externa, diagnostics);
    }
}

fn scan_block_for_go_unsupported_errors(
    block: &BlockStmt,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in &block.stmts {
        scan_stmt_for_go_unsupported_errors(stmt, file, dynamic_externa, diagnostics);
    }
}

fn scan_if_body_for_go_unsupported_errors(
    body: &IfBody,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match body {
        IfBody::Block(block) => scan_block_for_go_unsupported_errors(block, file, dynamic_externa, diagnostics),
        IfBody::Ergo(stmt) => scan_stmt_for_go_unsupported_errors(stmt, file, dynamic_externa, diagnostics),
        IfBody::InlineReturn(ret) => {
            scan_inline_return_for_go_unsupported_errors(ret, file, dynamic_externa, diagnostics)
        }
    }
}

fn scan_else_for_go_unsupported_errors(
    clause: &SecusClause,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match clause {
        SecusClause::Sin(stmt) => {
            scan_expr_for_go_unsupported_errors(&stmt.cond, file, dynamic_externa, diagnostics);
            scan_if_body_for_go_unsupported_errors(&stmt.then, file, dynamic_externa, diagnostics);
            if let Some(catch) = &stmt.catch {
                scan_block_for_go_unsupported_errors(&catch.body, file, dynamic_externa, diagnostics);
            }
            if let Some(else_) = &stmt.else_ {
                scan_else_for_go_unsupported_errors(else_, file, dynamic_externa, diagnostics);
            }
        }
        SecusClause::Block(block) => scan_block_for_go_unsupported_errors(block, file, dynamic_externa, diagnostics),
        SecusClause::Stmt(stmt) => scan_stmt_for_go_unsupported_errors(stmt, file, dynamic_externa, diagnostics),
        SecusClause::InlineReturn(ret) => {
            scan_inline_return_for_go_unsupported_errors(ret, file, dynamic_externa, diagnostics)
        }
    }
}

fn scan_inline_return_for_go_unsupported_errors(
    ret: &InlineReturn,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match ret {
        InlineReturn::Reddit(expr) | InlineReturn::Iacit(expr) | InlineReturn::Moritor(expr) => {
            scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics);
        }
        InlineReturn::Tacet => {}
    }
}

fn scan_expr_for_go_unsupported_errors(
    expr: &Expr,
    file: &str,
    dynamic_externa: &GoDynamicExterna,
    diagnostics: &mut Vec<Diagnostic>,
) {
    use crate::syntax::{
        ArrayElement, ClausuraBody, CollectionFilterKind, ExprKind, NonNullKind, ObjectKey, OptionalChainKind,
        PraefixumBody,
    };

    match &expr.kind {
        ExprKind::Binary(binary) => {
            scan_expr_for_go_unsupported_errors(&binary.lhs, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&binary.rhs, file, dynamic_externa, diagnostics);
        }
        ExprKind::Unary(unary) => scan_expr_for_go_unsupported_errors(&unary.operand, file, dynamic_externa, diagnostics),
        ExprKind::Call(call) => {
            scan_expr_for_go_unsupported_errors(&call.callee, file, dynamic_externa, diagnostics);
            for arg in &call.args {
                scan_expr_for_go_unsupported_errors(&arg.value, file, dynamic_externa, diagnostics);
            }
        }
        ExprKind::Member(member) => {
            if is_first_go_dynamic_externa_projection(&member.object, dynamic_externa) {
                diagnostics.push(go_target_policy_diagnostic(
                    file,
                    expr.span,
                    "member access on @ externa ignotum is not supported for Go targets",
                    "give the external binding or function a concrete Go-compatible type before projecting members, or target TypeScript/Bun for dynamic host objects",
                ));
            }
            scan_expr_for_go_unsupported_errors(&member.object, file, dynamic_externa, diagnostics);
        }
        ExprKind::Index(index) => {
            if is_first_go_dynamic_externa_projection(&index.object, dynamic_externa) {
                diagnostics.push(go_target_policy_diagnostic(
                    file,
                    expr.span,
                    "index access on @ externa ignotum is not supported for Go targets",
                    "cast the external value to a concrete Go-compatible collection type before indexing, or target TypeScript/Bun for dynamic host objects",
                ));
            }
            scan_expr_for_go_unsupported_errors(&index.object, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&index.index, file, dynamic_externa, diagnostics);
        }
        ExprKind::OptionalChain(chain) => {
            if is_first_go_dynamic_externa_projection(&chain.object, dynamic_externa) {
                diagnostics.push(go_target_policy_diagnostic(
                    file,
                    expr.span,
                    "optional chaining on @ externa ignotum is not supported for Go targets",
                    "cast the external value to a concrete Go-compatible type before optional access, or target TypeScript/Bun for dynamic host objects",
                ));
            }
            scan_expr_for_go_unsupported_errors(&chain.object, file, dynamic_externa, diagnostics);
            match &chain.chain {
                OptionalChainKind::Member(_) => {}
                OptionalChainKind::Index(index) => scan_expr_for_go_unsupported_errors(index, file, dynamic_externa, diagnostics),
                OptionalChainKind::Call(args) => {
                    for arg in args {
                        scan_expr_for_go_unsupported_errors(&arg.value, file, dynamic_externa, diagnostics);
                    }
                }
            }
        }
        ExprKind::NonNull(chain) => {
            if is_first_go_dynamic_externa_projection(&chain.object, dynamic_externa) {
                diagnostics.push(go_target_policy_diagnostic(
                    file,
                    expr.span,
                    "non-null projection on @ externa ignotum is not supported for Go targets",
                    "cast the external value to a concrete Go-compatible type before projecting members, or target TypeScript/Bun for dynamic host objects",
                ));
            }
            scan_expr_for_go_unsupported_errors(&chain.object, file, dynamic_externa, diagnostics);
            match &chain.chain {
                NonNullKind::Member(_) => {}
                NonNullKind::Index(index) => scan_expr_for_go_unsupported_errors(index, file, dynamic_externa, diagnostics),
                NonNullKind::Call(args) => {
                    for arg in args {
                        scan_expr_for_go_unsupported_errors(&arg.value, file, dynamic_externa, diagnostics);
                    }
                }
            }
        }
        ExprKind::Assign(assign) => {
            scan_expr_for_go_unsupported_errors(&assign.target, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&assign.value, file, dynamic_externa, diagnostics);
        }
        ExprKind::Ternary(ternary) => {
            scan_expr_for_go_unsupported_errors(&ternary.cond, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&ternary.then, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&ternary.else_, file, dynamic_externa, diagnostics);
        }
        ExprKind::Array(items) => {
            for item in &items.elements {
                match item {
                    ArrayElement::Expr(expr) | ArrayElement::Spread(expr) => {
                        scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics);
                    }
                }
            }
        }
        ExprKind::Object(fields) => {
            for field in &fields.fields {
                match &field.key {
                    ObjectKey::Computed(expr) | ObjectKey::Spread(expr) => {
                        scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics);
                    }
                    ObjectKey::Ident(_) | ObjectKey::String(_) => {}
                }
                if let Some(value) = &field.value {
                    scan_expr_for_go_unsupported_errors(value, file, dynamic_externa, diagnostics);
                }
            }
        }
        ExprKind::Scriptum(scriptum) => {
            for arg in &scriptum.args {
                scan_expr_for_go_unsupported_errors(arg, file, dynamic_externa, diagnostics);
            }
        }
        ExprKind::Ab(ab) => {
            scan_expr_for_go_unsupported_errors(&ab.source, file, dynamic_externa, diagnostics);
            if let Some(filter) = &ab.filter {
                match &filter.kind {
                    CollectionFilterKind::Condition(pred) => {
                        scan_expr_for_go_unsupported_errors(pred, file, dynamic_externa, diagnostics);
                    }
                    CollectionFilterKind::Property(_) => {}
                }
            }
            for transform in &ab.transforms {
                if let Some(arg) = &transform.arg {
                    scan_expr_for_go_unsupported_errors(arg, file, dynamic_externa, diagnostics);
                }
            }
        }
        ExprKind::Clausura(clausura) => match &clausura.body {
            ClausuraBody::Expr(expr) => scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics),
            ClausuraBody::Block(block) => scan_block_for_go_unsupported_errors(block, file, dynamic_externa, diagnostics),
        },
        ExprKind::Conversio(conversio) => {
            scan_expr_for_go_unsupported_errors(&conversio.expr, file, dynamic_externa, diagnostics);
            if let Some(fallback) = &conversio.fallback {
                scan_expr_for_go_unsupported_errors(fallback, file, dynamic_externa, diagnostics);
            }
        }
        ExprKind::Cede(cede) => scan_expr_for_go_unsupported_errors(&cede.expr, file, dynamic_externa, diagnostics),
        ExprKind::Verte(verte) => scan_expr_for_go_unsupported_errors(&verte.expr, file, dynamic_externa, diagnostics),
        ExprKind::Intervallum(range) => {
            scan_expr_for_go_unsupported_errors(&range.start, file, dynamic_externa, diagnostics);
            scan_expr_for_go_unsupported_errors(&range.end, file, dynamic_externa, diagnostics);
            if let Some(step) = &range.step {
                scan_expr_for_go_unsupported_errors(step, file, dynamic_externa, diagnostics);
            }
        }
        ExprKind::Finge(finge) => {
            for field in &finge.fields {
                scan_expr_for_go_unsupported_errors(&field.value, file, dynamic_externa, diagnostics);
            }
        }
        ExprKind::Praefixum(praefixum) => match &praefixum.body {
            PraefixumBody::Block(block) => scan_block_for_go_unsupported_errors(block, file, dynamic_externa, diagnostics),
            PraefixumBody::Expr(expr) => scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics),
        },
        ExprKind::Paren(expr) => scan_expr_for_go_unsupported_errors(expr, file, dynamic_externa, diagnostics),
        ExprKind::Literal(_) | ExprKind::Ident(_) | ExprKind::Lege(_) | ExprKind::Sed(_) | ExprKind::Ego(_) => {}
    }
}

fn is_first_go_dynamic_externa_projection(expr: &Expr, dynamic_externa: &GoDynamicExterna) -> bool {
    if matches!(
        expr.kind,
        ExprKind::Member(_) | ExprKind::Index(_) | ExprKind::OptionalChain(_) | ExprKind::NonNull(_)
    ) {
        return false;
    }

    matches!(go_dynamic_externa_root(expr, dynamic_externa), Some(_))
}

fn go_dynamic_externa_root(expr: &Expr, dynamic_externa: &GoDynamicExterna) -> Option<()> {
    match &expr.kind {
        ExprKind::Ident(ident) if dynamic_externa.bindings.contains(&ident.name) => Some(()),
        ExprKind::Call(call) => match &call.callee.kind {
            ExprKind::Ident(ident) if dynamic_externa.functions.contains(&ident.name) => Some(()),
            _ => None,
        },
        ExprKind::Member(member) => go_dynamic_externa_root(&member.object, dynamic_externa),
        ExprKind::Index(index) => go_dynamic_externa_root(&index.object, dynamic_externa),
        ExprKind::OptionalChain(chain) => go_dynamic_externa_root(&chain.object, dynamic_externa),
        ExprKind::NonNull(chain) => go_dynamic_externa_root(&chain.object, dynamic_externa),
        ExprKind::Paren(inner) => go_dynamic_externa_root(inner, dynamic_externa),
        _ => None,
    }
}

fn go_target_policy_diagnostic(file: &str, span: crate::lexer::Span, message: &str, help: &str) -> Diagnostic {
    Diagnostic::error(message)
        .with_code("TARGETGO001")
        .with_file(file)
        .with_span(span)
        .with_help(help)
}

fn scan_stmt_for_rust_unsupported_errors(stmt: &Stmt, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match &stmt.kind {
        StmtKind::Block(block) => scan_block_for_rust_unsupported_errors(block, file, diagnostics),
        StmtKind::Func(func) => {
            if let Some(body) = &func.body {
                scan_block_for_rust_unsupported_errors(body, file, diagnostics);
            }
        }
        StmtKind::Class(class) => {
            for member in &class.members {
                if let ClassMemberKind::Method(method) = &member.kind {
                    if let Some(body) = &method.body {
                        scan_block_for_rust_unsupported_errors(body, file, diagnostics);
                    }
                }
            }
        }
        StmtKind::Probandum(test) => {
            for setup in &test.body.setup {
                scan_block_for_rust_unsupported_errors(&setup.body, file, diagnostics);
            }
            for case in &test.body.tests {
                scan_block_for_rust_unsupported_errors(&case.body, file, diagnostics);
            }
            for nested in &test.body.nested {
                scan_probandum_for_rust_unsupported_errors(nested, file, diagnostics);
            }
        }
        StmtKind::Proba(test) => scan_block_for_rust_unsupported_errors(&test.body, file, diagnostics),
        StmtKind::Si(if_stmt) => {
            scan_expr_for_rust_unsupported_errors(&if_stmt.cond, file, diagnostics);
            scan_if_body_for_rust_unsupported_errors(&if_stmt.then, file, diagnostics);
            if let Some(catch) = &if_stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
            if let Some(else_) = &if_stmt.else_ {
                scan_else_for_rust_unsupported_errors(else_, file, diagnostics);
            }
        }
        StmtKind::Dum(while_stmt) => {
            scan_expr_for_rust_unsupported_errors(&while_stmt.cond, file, diagnostics);
            scan_if_body_for_rust_unsupported_errors(&while_stmt.body, file, diagnostics);
            if let Some(catch) = &while_stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Itera(iter_stmt) => {
            scan_expr_for_rust_unsupported_errors(&iter_stmt.iterable, file, diagnostics);
            scan_if_body_for_rust_unsupported_errors(&iter_stmt.body, file, diagnostics);
            if let Some(catch) = &iter_stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Elige(switch_stmt) => {
            scan_expr_for_rust_unsupported_errors(&switch_stmt.expr, file, diagnostics);
            for case in &switch_stmt.cases {
                scan_expr_for_rust_unsupported_errors(&case.value, file, diagnostics);
                scan_if_body_for_rust_unsupported_errors(&case.body, file, diagnostics);
            }
            if let Some(default) = &switch_stmt.default {
                scan_if_body_for_rust_unsupported_errors(&default.body, file, diagnostics);
            }
            if let Some(catch) = &switch_stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Discerne(match_stmt) => {
            for subject in &match_stmt.subjects {
                scan_expr_for_rust_unsupported_errors(subject, file, diagnostics);
            }
            for arm in &match_stmt.arms {
                scan_if_body_for_rust_unsupported_errors(&arm.body, file, diagnostics);
            }
            if let Some(default) = &match_stmt.default {
                scan_if_body_for_rust_unsupported_errors(&default.body, file, diagnostics);
            }
        }
        StmtKind::Custodi(guard_stmt) => {
            for clause in &guard_stmt.clauses {
                scan_expr_for_rust_unsupported_errors(&clause.cond, file, diagnostics);
                scan_if_body_for_rust_unsupported_errors(&clause.body, file, diagnostics);
            }
        }
        StmtKind::Fac(fac_stmt) => {
            scan_block_for_rust_unsupported_errors(&fac_stmt.body, file, diagnostics);
            if let Some(catch) = &fac_stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
            if let Some(cond) = &fac_stmt.while_ {
                scan_expr_for_rust_unsupported_errors(cond, file, diagnostics);
            }
        }
        StmtKind::Redde(ret) => {
            if let Some(value) = &ret.value {
                scan_expr_for_rust_unsupported_errors(value, file, diagnostics);
            }
        }
        StmtKind::Iace(stmt) => {
            diagnostics.push(rust_target_exception_diagnostic(
                file,
                stmt.value.span,
                "iace is not supported for Rust targets",
            ));
            scan_expr_for_rust_unsupported_errors(&stmt.value, file, diagnostics);
        }
        StmtKind::Mori(stmt) => scan_expr_for_rust_unsupported_errors(&stmt.value, file, diagnostics),
        StmtKind::Tempta(stmt) => {
            diagnostics.push(rust_target_exception_diagnostic(
                file,
                stmt.body.span,
                "tempta is not supported for Rust targets",
            ));
            scan_block_for_rust_unsupported_errors(&stmt.body, file, diagnostics);
            if let Some(catch) = &stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
            if let Some(finally) = &stmt.finally {
                scan_block_for_rust_unsupported_errors(finally, file, diagnostics);
            }
        }
        StmtKind::Adfirma(stmt) => {
            scan_expr_for_rust_unsupported_errors(&stmt.cond, file, diagnostics);
            if let Some(message) = &stmt.message {
                scan_expr_for_rust_unsupported_errors(message, file, diagnostics);
            }
        }
        StmtKind::Scribe(stmt) => {
            for arg in &stmt.args {
                scan_expr_for_rust_unsupported_errors(arg, file, diagnostics);
            }
        }
        StmtKind::Incipit(entry) => {
            if let Some(exitus) = &entry.exitus {
                scan_expr_for_rust_unsupported_errors(exitus, file, diagnostics);
            }
            scan_if_body_for_rust_unsupported_errors(&entry.body, file, diagnostics);
        }
        StmtKind::Cura(resource) => {
            if let Some(init) = &resource.init {
                scan_expr_for_rust_unsupported_errors(init, file, diagnostics);
            }
            scan_block_for_rust_unsupported_errors(&resource.body, file, diagnostics);
            if let Some(catch) = &resource.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Ad(endpoint) => {
            for arg in &endpoint.args {
                scan_expr_for_rust_unsupported_errors(&arg.value, file, diagnostics);
            }
            if let Some(body) = &endpoint.body {
                scan_block_for_rust_unsupported_errors(body, file, diagnostics);
            }
            if let Some(catch) = &endpoint.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
        }
        StmtKind::Expr(expr) => scan_expr_for_rust_unsupported_errors(&expr.expr, file, diagnostics),
        StmtKind::Var(decl) => {
            if let Some(init) = &decl.init {
                scan_expr_for_rust_unsupported_errors(init, file, diagnostics);
            }
        }
        StmtKind::Import(_)
        | StmtKind::TypeAlias(_)
        | StmtKind::Enum(_)
        | StmtKind::Union(_)
        | StmtKind::Interface(_)
        | StmtKind::Rumpe(_)
        | StmtKind::Perge(_) => {}
        StmtKind::Ex(stmt) => scan_expr_for_rust_unsupported_errors(&stmt.source, file, diagnostics),
    }
}

fn scan_probandum_for_rust_unsupported_errors(test: &ProbandumDecl, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    for setup in &test.body.setup {
        scan_block_for_rust_unsupported_errors(&setup.body, file, diagnostics);
    }
    for case in &test.body.tests {
        scan_block_for_rust_unsupported_errors(&case.body, file, diagnostics);
    }
    for nested in &test.body.nested {
        scan_probandum_for_rust_unsupported_errors(nested, file, diagnostics);
    }
}

fn scan_block_for_rust_unsupported_errors(block: &BlockStmt, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    for stmt in &block.stmts {
        scan_stmt_for_rust_unsupported_errors(stmt, file, diagnostics);
    }
}

fn scan_if_body_for_rust_unsupported_errors(body: &IfBody, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match body {
        IfBody::Block(block) => scan_block_for_rust_unsupported_errors(block, file, diagnostics),
        IfBody::Ergo(stmt) => scan_stmt_for_rust_unsupported_errors(stmt, file, diagnostics),
        IfBody::InlineReturn(ret) => scan_inline_return_for_rust_unsupported_errors(ret, file, diagnostics),
    }
}

fn scan_else_for_rust_unsupported_errors(clause: &SecusClause, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match clause {
        SecusClause::Sin(stmt) => {
            scan_expr_for_rust_unsupported_errors(&stmt.cond, file, diagnostics);
            scan_if_body_for_rust_unsupported_errors(&stmt.then, file, diagnostics);
            if let Some(catch) = &stmt.catch {
                diagnostics.push(rust_target_exception_diagnostic(
                    file,
                    catch.span,
                    "cape is not supported for Rust targets",
                ));
                scan_block_for_rust_unsupported_errors(&catch.body, file, diagnostics);
            }
            if let Some(else_) = &stmt.else_ {
                scan_else_for_rust_unsupported_errors(else_, file, diagnostics);
            }
        }
        SecusClause::Block(block) => scan_block_for_rust_unsupported_errors(block, file, diagnostics),
        SecusClause::Stmt(stmt) => scan_stmt_for_rust_unsupported_errors(stmt, file, diagnostics),
        SecusClause::InlineReturn(ret) => scan_inline_return_for_rust_unsupported_errors(ret, file, diagnostics),
    }
}

fn scan_inline_return_for_rust_unsupported_errors(ret: &InlineReturn, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match ret {
        InlineReturn::Reddit(expr) | InlineReturn::Moritor(expr) => {
            scan_expr_for_rust_unsupported_errors(expr, file, diagnostics);
        }
        InlineReturn::Iacit(expr) => {
            diagnostics.push(rust_target_exception_diagnostic(
                file,
                expr.span,
                "iacit is not supported for Rust targets",
            ));
            scan_expr_for_rust_unsupported_errors(expr, file, diagnostics);
        }
        InlineReturn::Tacet => {}
    }
}

fn scan_expr_for_rust_unsupported_errors(expr: &Expr, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    use crate::syntax::{
        ArrayElement, ClausuraBody, CollectionFilterKind, ExprKind, NonNullKind, ObjectKey, OptionalChainKind,
        PraefixumBody,
    };

    match &expr.kind {
        ExprKind::Binary(binary) => {
            scan_expr_for_rust_unsupported_errors(&binary.lhs, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&binary.rhs, file, diagnostics);
        }
        ExprKind::Unary(unary) => scan_expr_for_rust_unsupported_errors(&unary.operand, file, diagnostics),
        ExprKind::Call(call) => {
            scan_expr_for_rust_unsupported_errors(&call.callee, file, diagnostics);
            for arg in &call.args {
                scan_expr_for_rust_unsupported_errors(&arg.value, file, diagnostics);
            }
        }
        ExprKind::Member(member) => scan_expr_for_rust_unsupported_errors(&member.object, file, diagnostics),
        ExprKind::Index(index) => {
            scan_expr_for_rust_unsupported_errors(&index.object, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&index.index, file, diagnostics);
        }
        ExprKind::OptionalChain(chain) => {
            scan_expr_for_rust_unsupported_errors(&chain.object, file, diagnostics);
            match &chain.chain {
                OptionalChainKind::Member(_) => {}
                OptionalChainKind::Index(index) => scan_expr_for_rust_unsupported_errors(index, file, diagnostics),
                OptionalChainKind::Call(args) => {
                    for arg in args {
                        scan_expr_for_rust_unsupported_errors(&arg.value, file, diagnostics);
                    }
                }
            }
        }
        ExprKind::NonNull(chain) => {
            scan_expr_for_rust_unsupported_errors(&chain.object, file, diagnostics);
            match &chain.chain {
                NonNullKind::Member(_) => {}
                NonNullKind::Index(index) => scan_expr_for_rust_unsupported_errors(index, file, diagnostics),
                NonNullKind::Call(args) => {
                    for arg in args {
                        scan_expr_for_rust_unsupported_errors(&arg.value, file, diagnostics);
                    }
                }
            }
        }
        ExprKind::Assign(assign) => {
            scan_expr_for_rust_unsupported_errors(&assign.target, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&assign.value, file, diagnostics);
        }
        ExprKind::Ternary(ternary) => {
            scan_expr_for_rust_unsupported_errors(&ternary.cond, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&ternary.then, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&ternary.else_, file, diagnostics);
        }
        ExprKind::Array(items) => {
            for item in &items.elements {
                match item {
                    ArrayElement::Expr(expr) | ArrayElement::Spread(expr) => {
                        scan_expr_for_rust_unsupported_errors(expr, file, diagnostics);
                    }
                }
            }
        }
        ExprKind::Object(fields) => {
            for field in &fields.fields {
                match &field.key {
                    ObjectKey::Computed(expr) | ObjectKey::Spread(expr) => {
                        scan_expr_for_rust_unsupported_errors(expr, file, diagnostics);
                    }
                    ObjectKey::Ident(_) | ObjectKey::String(_) => {}
                }
                if let Some(value) = &field.value {
                    scan_expr_for_rust_unsupported_errors(value, file, diagnostics);
                }
            }
        }
        ExprKind::Scriptum(scriptum) => {
            for arg in &scriptum.args {
                scan_expr_for_rust_unsupported_errors(arg, file, diagnostics);
            }
        }
        ExprKind::Ab(ab) => {
            scan_expr_for_rust_unsupported_errors(&ab.source, file, diagnostics);
            if let Some(filter) = &ab.filter {
                match &filter.kind {
                    CollectionFilterKind::Condition(pred) => {
                        scan_expr_for_rust_unsupported_errors(pred, file, diagnostics);
                    }
                    CollectionFilterKind::Property(_) => {}
                }
            }
            for transform in &ab.transforms {
                if let Some(arg) = &transform.arg {
                    scan_expr_for_rust_unsupported_errors(arg, file, diagnostics);
                }
            }
        }
        ExprKind::Clausura(clausura) => match &clausura.body {
            ClausuraBody::Expr(expr) => scan_expr_for_rust_unsupported_errors(expr, file, diagnostics),
            ClausuraBody::Block(block) => scan_block_for_rust_unsupported_errors(block, file, diagnostics),
        },
        ExprKind::Conversio(conversio) => {
            scan_expr_for_rust_unsupported_errors(&conversio.expr, file, diagnostics);
            if let Some(fallback) = &conversio.fallback {
                scan_expr_for_rust_unsupported_errors(fallback, file, diagnostics);
            }
        }
        ExprKind::Cede(cede) => scan_expr_for_rust_unsupported_errors(&cede.expr, file, diagnostics),
        ExprKind::Verte(verte) => scan_expr_for_rust_unsupported_errors(&verte.expr, file, diagnostics),
        ExprKind::Intervallum(range) => {
            scan_expr_for_rust_unsupported_errors(&range.start, file, diagnostics);
            scan_expr_for_rust_unsupported_errors(&range.end, file, diagnostics);
            if let Some(step) = &range.step {
                scan_expr_for_rust_unsupported_errors(step, file, diagnostics);
            }
        }
        ExprKind::Finge(finge) => {
            for field in &finge.fields {
                scan_expr_for_rust_unsupported_errors(&field.value, file, diagnostics);
            }
        }
        ExprKind::Praefixum(praefixum) => match &praefixum.body {
            PraefixumBody::Block(block) => scan_block_for_rust_unsupported_errors(block, file, diagnostics),
            PraefixumBody::Expr(expr) => scan_expr_for_rust_unsupported_errors(expr, file, diagnostics),
        },
        ExprKind::Paren(expr) => scan_expr_for_rust_unsupported_errors(expr, file, diagnostics),
        ExprKind::Literal(_) | ExprKind::Ident(_) | ExprKind::Lege(_) | ExprKind::Sed(_) | ExprKind::Ego(_) => {}
    }
}

fn rust_target_exception_diagnostic(file: &str, span: crate::lexer::Span, message: &str) -> Diagnostic {
    Diagnostic::error(message)
        .with_code("TARGET001")
        .with_file(file)
        .with_span(span)
        .with_help("use 'mori'/'moritor' for panic-style aborts, or target Faber/TypeScript for exception flow")
}

/// Collect warnings for Rust-specific no-ops.
///
/// WHY: Some Faber constructs are meaningful in GC'd targets (e.g., JavaScript,
/// Python) but no-ops in Rust (e.g., `cura arena`). We warn users to avoid
/// confusion about why arena scoping has no effect.
fn collect_rust_noop_warnings(program: &Program, file: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for stmt in &program.stmts {
        scan_stmt_for_rust_warnings(stmt, file, &mut diagnostics);
    }
    diagnostics
}

/// Recursively scan statements for Rust no-op warnings.
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
            // WHY: Arena resource management is a no-op in Rust (RAII handles it)
            if matches!(resource.kind, Some(CuraKind::Arena)) {
                let spec = crate::diagnostics::semantic_spec(crate::semantic::SemanticErrorKind::Warning(
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

fn scan_test_for_rust_warnings(test: &ProbandumDecl, file: &str, diagnostics: &mut Vec<Diagnostic>) {
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

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
