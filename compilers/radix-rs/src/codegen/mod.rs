//! Code generation from HIR to target languages
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! The codegen module transforms HIR (High-level Intermediate Representation) into
//! executable source code for target languages. This is the final phase of the
//! compilation pipeline.
//!
//! COMPILER PHASE: Codegen
//! INPUT: HirProgram (semantic analysis output), TypeTable, Interner
//! OUTPUT: Target-specific source code (Rust, Faber pretty-print, or future targets)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Target abstraction: Each backend implements the `Codegen` trait, allowing
//!   uniform access regardless of target language. New backends can be added
//!   without modifying the driver logic.
//!
//! - Multi-target support: Different backends handle language-specific quirks
//!   (Rust's Result wrapping for failable functions, Faber's Latin keywords, etc.)
//!   through specialized transforms.
//!
//! - Error propagation: Failable functions (those that contain `iace` / throw
//!   expressions) are detected during Rust codegen and emit Result<T, String>
//!   signatures with automatic `?` operator insertion.
//!
//! BACKENDS
//! ========
//! - Rust: Full compilation to executable Rust code
//! - Faber: Canonical pretty-printing for formatting and round-tripping

pub mod faber;
pub mod rust;
pub mod ts;
mod writer;

pub use writer::CodeWriter;

use crate::hir::HirProgram;
use crate::lexer::Interner;
use crate::semantic::TypeTable;

// =============================================================================
// TYPES
// =============================================================================
//
// Core types for target specification and error handling.

/// Compilation target language.
///
/// WHY: Target enumeration allows the driver to select backends without
/// hardcoding backend names throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
    Faber,
    TypeScript,
}

/// Code generation error.
///
/// WHY: Codegen errors are distinct from semantic errors because they occur
/// after all semantic analysis passes. They typically indicate unimplemented
/// features or internal compiler bugs rather than user code errors.
#[derive(Debug)]
pub struct CodegenError {
    pub message: String,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CodegenError {}

// =============================================================================
// CORE
// =============================================================================
//
// Main codegen trait and dispatch logic.

/// Code generation trait for different targets.
///
/// WHY: This trait enables target-agnostic driver code. Each backend provides
/// its own implementation with target-specific transforms and conventions.
pub trait Codegen {
    type Output;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<Self::Output, CodegenError>;
}

/// Generate code for the specified target.
///
/// WHY: This function provides a unified entry point for all codegen backends,
/// dispatching to the appropriate implementation based on the target enum.
///
/// TRANSFORMS: Delegates to target-specific generators that handle language
/// quirks (e.g., Rust Result wrapping, Faber Latin keywords).
pub fn generate(
    target: Target,
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
) -> Result<crate::Output, CodegenError> {
    reject_hir_errors(hir)?;

    match target {
        Target::Rust => {
            let gen = rust::RustCodegen::new(hir, interner);
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::Rust(output))
        }
        Target::Faber => {
            let gen = faber::FaberCodegen::new();
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::Faber(output))
        }
        Target::TypeScript => {
            let gen = ts::TsCodegen::new(hir, interner);
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::TypeScript(output))
        }
    }
}

pub(super) fn reject_hir_errors(hir: &HirProgram) -> Result<(), CodegenError> {
    if let Some(span) = find_error_expr_in_program(hir) {
        return Err(CodegenError {
            message: format!(
                "cannot generate code from HIR containing error expressions at span {}..{}",
                span.start, span.end
            ),
        });
    }
    Ok(())
}

fn find_error_expr_in_program(hir: &HirProgram) -> Option<crate::lexer::Span> {
    for item in &hir.items {
        if let Some(span) = find_error_expr_in_item(item) {
            return Some(span);
        }
    }

    if let Some(entry) = &hir.entry {
        return find_error_expr_in_block(entry);
    }

    None
}

fn find_error_expr_in_item(item: &crate::hir::HirItem) -> Option<crate::lexer::Span> {
    match &item.kind {
        crate::hir::HirItemKind::Function(func) => func.body.as_ref().and_then(find_error_expr_in_block),
        crate::hir::HirItemKind::Struct(struct_item) => {
            for field in &struct_item.fields {
                if let Some(init) = &field.init {
                    if let Some(span) = find_error_expr_in_expr(init) {
                        return Some(span);
                    }
                }
            }
            for method in &struct_item.methods {
                if let Some(body) = &method.func.body {
                    if let Some(span) = find_error_expr_in_block(body) {
                        return Some(span);
                    }
                }
            }
            None
        }
        crate::hir::HirItemKind::Const(const_item) => find_error_expr_in_expr(&const_item.value),
        _ => None,
    }
}

fn find_error_expr_in_block(block: &crate::hir::HirBlock) -> Option<crate::lexer::Span> {
    for stmt in &block.stmts {
        if let Some(span) = find_error_expr_in_stmt(stmt) {
            return Some(span);
        }
    }

    block
        .expr
        .as_ref()
        .and_then(|expr| find_error_expr_in_expr(expr))
}

fn find_error_expr_in_stmt(stmt: &crate::hir::HirStmt) -> Option<crate::lexer::Span> {
    match &stmt.kind {
        crate::hir::HirStmtKind::Local(local) => local.init.as_ref().and_then(find_error_expr_in_expr),
        crate::hir::HirStmtKind::Expr(expr) => find_error_expr_in_expr(expr),
        crate::hir::HirStmtKind::Ad(ad) => ad
            .args
            .iter()
            .find_map(find_error_expr_in_expr)
            .or_else(|| ad.body.as_ref().and_then(find_error_expr_in_block))
            .or_else(|| ad.catch.as_ref().and_then(find_error_expr_in_block)),
        crate::hir::HirStmtKind::Redde(expr) => expr.as_ref().and_then(find_error_expr_in_expr),
        crate::hir::HirStmtKind::Rumpe | crate::hir::HirStmtKind::Perge => None,
    }
}

fn find_error_expr_in_expr(expr: &crate::hir::HirExpr) -> Option<crate::lexer::Span> {
    use crate::hir::{HirCollectionFilterKind, HirExprKind, HirNonNullKind, HirOptionalChainKind};

    match &expr.kind {
        HirExprKind::Error => Some(expr.span),
        HirExprKind::Path(_) | HirExprKind::Literal(_) => None,
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            find_error_expr_in_expr(lhs).or_else(|| find_error_expr_in_expr(rhs))
        }
        HirExprKind::Unary(_, inner)
        | HirExprKind::Field(inner, _)
        | HirExprKind::Panic(inner)
        | HirExprKind::Throw(inner)
        | HirExprKind::Cede(inner)
        | HirExprKind::Ref(_, inner)
        | HirExprKind::Deref(inner) => find_error_expr_in_expr(inner),
        HirExprKind::Call(callee, args) | HirExprKind::MethodCall(callee, _, args) => {
            find_error_expr_in_expr(callee).or_else(|| args.iter().find_map(find_error_expr_in_expr))
        }
        HirExprKind::Index(object, index) => find_error_expr_in_expr(object).or_else(|| find_error_expr_in_expr(index)),
        HirExprKind::OptionalChain(object, chain) => find_error_expr_in_expr(object).or_else(|| match chain {
            HirOptionalChainKind::Member(_) => None,
            HirOptionalChainKind::Index(index) => find_error_expr_in_expr(index),
            HirOptionalChainKind::Call(args) => args.iter().find_map(find_error_expr_in_expr),
        }),
        HirExprKind::NonNull(object, chain) => find_error_expr_in_expr(object).or_else(|| match chain {
            HirNonNullKind::Member(_) => None,
            HirNonNullKind::Index(index) => find_error_expr_in_expr(index),
            HirNonNullKind::Call(args) => args.iter().find_map(find_error_expr_in_expr),
        }),
        HirExprKind::Ab { source, filter, transforms } => find_error_expr_in_expr(source)
            .or_else(|| {
                filter.as_ref().and_then(|filter| match &filter.kind {
                    HirCollectionFilterKind::Condition(condition) => find_error_expr_in_expr(condition),
                    HirCollectionFilterKind::Property(_) => None,
                })
            })
            .or_else(|| {
                transforms.iter().find_map(|transform| {
                    transform
                        .arg
                        .as_ref()
                        .and_then(|arg| find_error_expr_in_expr(arg))
                })
            }),
        HirExprKind::Block(block) | HirExprKind::Loop(block) => find_error_expr_in_block(block),
        HirExprKind::Si(cond, then_block, else_block) => find_error_expr_in_expr(cond)
            .or_else(|| find_error_expr_in_block(then_block))
            .or_else(|| else_block.as_ref().and_then(find_error_expr_in_block)),
        HirExprKind::Discerne(scrutinees, arms) => scrutinees
            .iter()
            .find_map(find_error_expr_in_expr)
            .or_else(|| {
                arms.iter().find_map(|arm| {
                    arm.guard
                        .as_ref()
                        .and_then(find_error_expr_in_expr)
                        .or_else(|| find_error_expr_in_expr(&arm.body))
                })
            }),
        HirExprKind::Dum(cond, block) => find_error_expr_in_expr(cond).or_else(|| find_error_expr_in_block(block)),
        HirExprKind::Itera(_, _, iter, block) => {
            find_error_expr_in_expr(iter).or_else(|| find_error_expr_in_block(block))
        }
        HirExprKind::Array(elements) | HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
            elements.iter().find_map(find_error_expr_in_expr)
        }
        HirExprKind::Struct(_, fields) => fields
            .iter()
            .find_map(|(_, value)| find_error_expr_in_expr(value)),
        HirExprKind::Scriptum(_, args) => args.iter().find_map(find_error_expr_in_expr),
        HirExprKind::Adfirma(cond, message) => find_error_expr_in_expr(cond).or_else(|| {
            message
                .as_ref()
                .and_then(|message| find_error_expr_in_expr(message))
        }),
        HirExprKind::Tempta { body, catch, finally } => find_error_expr_in_block(body)
            .or_else(|| catch.as_ref().and_then(find_error_expr_in_block))
            .or_else(|| finally.as_ref().and_then(find_error_expr_in_block)),
        HirExprKind::Clausura(_, _, body) => find_error_expr_in_expr(body),
        HirExprKind::Verte { source, entries, .. } => find_error_expr_in_expr(source).or_else(|| {
            entries.as_ref().and_then(|entries| {
                entries
                    .iter()
                    .find_map(|(_, value)| find_error_expr_in_expr(value))
            })
        }),
        HirExprKind::Conversio { source, fallback, .. } => {
            find_error_expr_in_expr(source).or_else(|| fallback.as_ref().and_then(|fb| find_error_expr_in_expr(fb)))
        }
    }
}
