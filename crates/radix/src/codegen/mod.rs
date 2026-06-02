//! Shared code generation boundary for HIR-backed target emitters.
//!
//! Codegen is the last HIR-facing compiler phase. Semantic analysis has already
//! produced a typed [`HirProgram`] and [`TypeTable`]; this module selects a
//! backend, rejects poisoned HIR that still contains recovery error nodes, and
//! returns the target-specific source artifact consumed by callers in `radix`
//! and the user-facing `faber` package builder.
//!
//! The per-target backends own language mapping policy. The shared layer owns
//! only contracts that every backend must obey: a common dispatch surface,
//! a single codegen error type, access to semantic types, and fail-closed
//! handling for HIR that semantic recovery could not make safe to emit.
//!
//! INVARIANTS
//! ==========
//! - Backends receive HIR only after [`reject_hir_errors`] proves that recovery
//!   expressions are absent.
//! - The shared [`Codegen`] trait does not prescribe formatting, naming, or
//!   runtime strategy; those are backend policies.
//! - [`CodegenError`] marks an emission boundary problem, not a replacement for
//!   semantic diagnostics that should have been collected earlier.
//! - Target dispatch is explicit so adding a backend must also choose its
//!   public [`crate::Output`] variant.

pub mod faber;
pub mod go;
mod names;
pub mod rust;
pub mod ts;
mod writer;

pub use writer::CodeWriter;

use crate::hir::HirProgram;
use crate::lexer::Interner;
use crate::semantic::TypeTable;

/// Compilation target selected after analysis and before backend dispatch.
///
/// This enum is the public routing surface for drivers. Each variant commits to
/// a concrete backend module and a matching [`crate::Output`] payload shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Native Rust source, used for package builds and executable output.
    Rust,

    /// Canonical Faber pretty-printer output for source-formatting workflows.
    Faber,

    /// TypeScript source for JavaScript ecosystem targets.
    TypeScript,

    /// Go source for experiments in a second systems/runtime target.
    Go,

    /// Experimental MIR-backed WebAssembly text probe.
    WasmText,

    /// Experimental MIR-backed LLVM text probe.
    LlvmText,
}

/// Backend emission error after semantic analysis has completed.
///
/// Codegen errors are intentionally narrower than semantic diagnostics. They
/// represent target gaps, unsupported lowering combinations, or internal
/// pipeline violations discovered while emitting source. User-facing type and
/// name errors should be reported before this boundary whenever possible.
#[derive(Debug)]
pub struct CodegenError {
    /// Human-readable emission failure suitable for CLI diagnostic wrapping.
    pub message: String,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CodegenError {}

/// Target backend contract for HIR-to-source emission.
///
/// Implementors receive the same semantic inputs: HIR for structure, the
/// [`TypeTable`] for resolved type decisions, and the [`Interner`] for
/// recovering source symbols. The output type stays associated so specialized
/// backends can return richer artifacts, while the public [`generate`] router
/// wraps those artifacts into [`crate::Output`].
pub trait Codegen {
    type Output;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<Self::Output, CodegenError>;
}

/// Dispatch HIR to the selected target backend.
///
/// The shared entrypoint rejects HIR recovery artifacts before backend code can
/// accidentally print placeholders into generated source. After that guard, all
/// language-specific choices are delegated to the backend selected by
/// [`Target`].
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
        Target::Go => {
            let gen = go::GoCodegen::new(hir, interner);
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::Go(output))
        }
        Target::WasmText | Target::LlvmText => {
            Err(CodegenError { message: "target is MIR-backed and must be routed through the driver".to_owned() })
        }
    }
}

pub fn generate_rust_cli(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    cli_program: &crate::cli::CliProgram,
) -> Result<crate::RustOutput, CodegenError> {
    reject_hir_errors(hir)?;
    rust::RustCodegen::new(hir, interner).generate_cli(hir, types, cli_program)
}

/// Reject semantically poisoned HIR before target emission begins.
///
/// HIR error expressions are recovery sentinels, not source constructs. Keeping
/// this guard at the codegen boundary lets parser/typechecker recovery collect
/// diagnostics earlier while still preventing downstream backends from turning
/// incomplete compiler state into misleading target code.
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
    let mut finder = ErrorExprFinder::default();
    crate::hir::visit::HirVisitor::visit_program(&mut finder, hir);
    finder.span
}

#[derive(Default)]
struct ErrorExprFinder {
    span: Option<crate::lexer::Span>,
}

impl crate::hir::visit::HirVisitor for ErrorExprFinder {
    fn visit_expr(&mut self, expr: &crate::hir::HirExpr) {
        if self.span.is_some() {
            return;
        }
        if matches!(expr.kind, crate::hir::HirExprKind::Error) {
            self.span = Some(expr.span);
            return;
        }
        crate::hir::visit::walk_expr(self, expr);
    }
}
