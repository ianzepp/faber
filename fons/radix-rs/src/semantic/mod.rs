//! Semantic analysis
//!
//! Multi-pass semantic analysis:
//! 1. Collect - gather all top-level declarations
//! 2. Resolve - resolve names to definitions
//! 3. Lower - build resolved HIR from the AST
//! 4. Typecheck - bidirectional type inference
//! 5. Borrow - ownership/borrowing analysis (for Rust target)
//! 6. Exhaustive - pattern match exhaustiveness
//! 7. Lint - warnings and suggestions

mod error;
pub mod passes;
mod scope;
mod types;

pub use error::{SemanticError, SemanticErrorKind, WarningKind};
pub use scope::{Resolver, Scope, ScopeId, ScopeKind, Symbol, SymbolKind};
pub use types::{FuncSig, Mutability, ParamMode, ParamType, Primitive, Type, TypeId, TypeTable};

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
        self.hir.is_some() && self.errors.is_empty()
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
            Target::Rust => Self {
                borrow_analysis: true,
                exhaustiveness: true,
                lint: true,
            },
            Target::Faber => Self {
                borrow_analysis: false,
                exhaustiveness: true,
                lint: true,
            },
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
        return SemanticResult {
            hir: None,
            types,
            errors,
        };
    }

    // Pass 3: Lower to HIR
    let (mut hir, lower_errors) = crate::hir::lower(program, &resolver, &mut types, interner);

    // Add lowering errors
    for err in lower_errors {
        errors.push(SemanticError::new(
            SemanticErrorKind::LoweringError,
            err.message,
            err.span,
        ));
    }

    if !errors.is_empty() {
        return SemanticResult {
            hir: None,
            types,
            errors,
        };
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

    SemanticResult {
        hir: Some(hir),
        types,
        errors,
    }
}
