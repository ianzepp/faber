//! Semantic analysis
//!
//! Multi-pass semantic analysis:
//! 1. Collect - gather all declarations
//! 2. Resolve - resolve names to definitions
//! 3. Typecheck - bidirectional type inference
//! 4. Borrow - ownership/borrowing analysis (for Rust target)
//! 5. Exhaustive - pattern match exhaustiveness
//! 6. Lint - warnings and suggestions

mod types;
mod scope;
mod error;
pub mod passes;

pub use types::{Type, TypeTable, Primitive, TypeId, Mutability, FuncSig, ParamType, ParamMode};
pub use scope::{Scope, ScopeId, ScopeKind, Symbol, SymbolKind, Resolver};
pub use error::{SemanticError, SemanticErrorKind};

use crate::syntax::Program;
use crate::hir::HirProgram;
use crate::codegen::Target;

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
pub fn analyze(program: &Program, config: &PassConfig) -> SemanticResult {
    let mut errors = Vec::new();
    let mut types = TypeTable::new();
    let mut resolver = Resolver::new();

    // Pass 1: Collect declarations
    if let Err(e) = passes::collect::collect(program, &mut resolver, &mut types) {
        errors.extend(e);
    }

    // Pass 2: Resolve names
    if let Err(e) = passes::resolve::resolve(program, &mut resolver, &mut types) {
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

    // Lower to HIR
    let hir = crate::hir::lower(program, &resolver);

    // Pass 3: Type checking
    if let Err(e) = passes::typecheck::typecheck(&hir, &resolver, &mut types) {
        errors.extend(e);
    }

    // Pass 4: Borrow analysis (conditional)
    if config.borrow_analysis {
        if let Err(e) = passes::borrow::analyze(&hir, &resolver, &types) {
            errors.extend(e);
        }
    }

    // Pass 5: Exhaustiveness checking (conditional)
    if config.exhaustiveness {
        if let Err(e) = passes::exhaustive::check(&hir, &types) {
            errors.extend(e);
        }
    }

    // Pass 6: Linting (conditional)
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
