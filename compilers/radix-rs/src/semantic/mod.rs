//! Semantic analysis
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Implements multi-pass semantic analysis transforming the AST into a typed,
//! validated HIR suitable for code generation. Each pass depends on the results
//! of previous passes, ensuring early errors prevent cascading failures.
//!
//! COMPILER PHASE: Semantic
//! INPUT: AST (syntax::Program) from parser
//! OUTPUT: HirProgram with type annotations and semantic errors
//!
//! MULTI-PASS PIPELINE
//! ===================
//! The analysis runs in this order:
//! 1. Collect - Gather all top-level declarations into symbol table
//! 2. Resolve - Resolve all name references to DefIds
//! 3. Lower - Transform AST into HIR with resolved names
//! 4. Typecheck - Bidirectional type inference and checking
//! 5. Borrow - Ownership/borrowing analysis (Rust target only)
//! 6. Exhaustive - Pattern match exhaustiveness checking
//! 7. Lint - Warnings and best practice suggestions
//!
//! WHY: Multi-pass design allows each pass to assume invariants from prior
//! passes (e.g., type checker assumes all names are resolved). Early exit on
//! errors prevents cascading failures and confusing error messages.
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Fail Fast: Collection and resolution errors stop the pipeline before
//!   lowering, avoiding complex error recovery in later passes
//! - Error Collection: Each pass collects all errors rather than stopping at
//!   the first, providing complete diagnostic information
//! - Target-Specific Analysis: Borrow checking only runs for Rust target,
//!   avoiding false positives for permissive targets like Faber pretty-print
//! - Configurable Passes: PassConfig allows disabling analysis for faster
//!   development iteration or target-specific needs
//!
//! ERROR HANDLING
//! ==============
//! Errors are collected into SemanticResult, which distinguishes hard errors
//! (prevent codegen) from warnings (informational). The success() method checks
//! for hard errors only, allowing compilation to proceed with warnings.

mod error;
pub mod passes;
mod scope;
mod types;

pub use error::{SemanticError, SemanticErrorKind, WarningKind};
pub use scope::{Resolver, Scope, ScopeId, ScopeKind, Symbol, SymbolKind};
pub use types::{
    CollectionKind, FuncSig, InferVar, Mutability, ParamMode, ParamType, Primitive, Type, TypeId, TypeTable,
};

use crate::codegen::Target;
use crate::hir::HirProgram;
use crate::lexer::Interner;
use crate::syntax::Program;

/// Semantic analysis result
pub struct SemanticResult {
    pub hir: Option<HirProgram>,
    pub types: TypeTable,
    pub errors: Vec<SemanticError>,
}

impl SemanticResult {
    pub fn success(&self) -> bool {
        self.hir.is_some() && !self.errors.iter().any(|err| err.is_error())
    }
}

/// Configuration for semantic passes
pub struct PassConfig {
    pub borrow_analysis: bool,
    pub exhaustiveness: bool,
    pub lint: bool,
}

impl PassConfig {
    pub fn for_target(target: Target) -> Self {
        match target {
            Target::Rust => Self { borrow_analysis: true, exhaustiveness: true, lint: true },
            Target::Faber => Self { borrow_analysis: false, exhaustiveness: true, lint: true },
            Target::TypeScript => Self { borrow_analysis: false, exhaustiveness: true, lint: true },
            Target::Go => Self { borrow_analysis: false, exhaustiveness: true, lint: true },
        }
    }
}

/// Run semantic analysis on a program
pub fn analyze(program: &Program, config: &PassConfig, interner: &Interner) -> SemanticResult {
    let mut errors = Vec::new();
    let mut types = TypeTable::new();
    let mut resolver = Resolver::new();

    // Pass 1: Collect declarations
    if let Err(e) = passes::collect::collect(program, &mut resolver, &mut types) {
        errors.extend(e);
    }

    // Pass 2: Resolve names
    if let Err(e) = passes::resolve::resolve(program, &mut resolver, interner, &mut types) {
        errors.extend(e);
    }

    // Early exit if resolution failed
    if !errors.is_empty() {
        return SemanticResult { hir: None, types, errors };
    }

    // Pass 3: Lower to HIR
    let (mut hir, lower_errors) = crate::hir::lower(program, &resolver, &mut types, interner);

    // Add lowering errors
    for err in lower_errors {
        errors.push(SemanticError::new(SemanticErrorKind::LoweringError, err.message, err.span));
    }

    if !errors.is_empty() {
        return SemanticResult { hir: None, types, errors };
    }

    // Pass 4: Type checking
    if let Err(e) = passes::typecheck::typecheck(&mut hir, &resolver, &mut types) {
        errors.extend(e);
    }

    // Pass 5: Borrow analysis (conditional)
    if config.borrow_analysis {
        if let Err(e) = passes::borrow::analyze(&hir, &resolver, &types) {
            errors.extend(e);
        }
    }

    // Pass 6: Exhaustiveness checking (conditional)
    if config.exhaustiveness {
        if let Err(e) = passes::exhaustive::check(&hir, &types) {
            errors.extend(e);
        }
    }

    // Pass 7: Linting (conditional)
    if config.lint {
        if let Err(e) = passes::lint::lint(&hir, &resolver, &types) {
            errors.extend(e);
        }
    }

    SemanticResult { hir: Some(hir), types, errors }
}
