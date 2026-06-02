//! Semantic orchestration for resolved, typed compiler state.
//!
//! This module is the boundary between parsed syntax and the HIR consumed by
//! backend-independent analysis and code generation. It owns the semantic pass
//! order, the shared resolver and type table, and the policy that decides when a
//! partially analyzed program is too compromised to lower further.
//!
//! PASS ORDER
//! ==========
//! - Collect top-level declarations into the resolver and seed type metadata.
//! - Resolve AST names against that resolver.
//! - Lower the resolved AST into HIR.
//! - Typecheck the HIR using the resolver and shared [`TypeTable`].
//! - Run configured non-typecheck analyses over HIR: borrow analysis,
//!   exhaustiveness, and linting.
//!
//! ERROR HANDLING
//! ==============
//! Collection, resolution, and HIR lowering errors stop the pipeline before
//! later passes would have to interpret invalid inputs. Typecheck and later
//! analyses collect diagnostics into one [`SemanticResult`]. Warnings flow
//! through the same vector as errors, but [`SemanticResult::success`] treats
//! [`SemanticErrorKind::Warning`] as non-fatal.
//!
//! INVARIANTS
//! ==========
//! - HIR lowering sits after AST resolve and before HIR typecheck.
//! - The resolver is the definition identity source for names created before
//!   lowering.
//! - The type table is shared across passes so `TypeId` values remain valid for
//!   the whole semantic result.
//! - Target-specific policy is expressed through [`PassConfig`]; this module
//!   only runs the passes that configuration enables.

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

/// Final semantic product for a parsed program.
///
/// `hir` is absent when an earlier hard diagnostic made downstream phases
/// unreliable. `types` is still returned so diagnostic consumers can inspect
/// whatever type state earlier passes established, and `errors` contains both
/// fatal diagnostics and non-fatal warnings.
pub struct SemanticResult {
    pub hir: Option<HirProgram>,
    pub types: TypeTable,
    pub errors: Vec<SemanticError>,
}

impl SemanticResult {
    /// Return whether this result is usable by later compiler stages.
    ///
    /// Warnings are intentionally non-fatal here; callers that want stricter
    /// policy must inspect `errors` themselves.
    pub fn success(&self) -> bool {
        self.hir.is_some() && !self.errors.iter().any(|err| err.is_error())
    }
}

/// Target and caller policy for optional semantic analyses.
///
/// Collection, name resolution, lowering, and typechecking are always part of
/// semantic analysis. These flags cover later HIR analyses whose value depends
/// on the target or caller: Rust enables borrow analysis, while other current
/// targets keep exhaustiveness and lint checks without enforcing Rust ownership
/// rules.
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
            Target::Wasm => Self { borrow_analysis: false, exhaustiveness: true, lint: true },
            Target::LlvmText => Self { borrow_analysis: false, exhaustiveness: true, lint: true },
        }
    }
}

/// Run semantic analysis on a parsed program without CLI mount context.
pub fn analyze(program: &Program, config: &PassConfig, interner: &Interner) -> SemanticResult {
    analyze_with_cli(program, config, interner, None)
}

/// Run the semantic pipeline, optionally validating mounted CLI declarations.
///
/// CLI metadata participates during HIR lowering because command mounts depend
/// on resolved declarations but become HIR-level entrypoint metadata for later
/// code generation.
pub fn analyze_with_cli(
    program: &Program,
    config: &PassConfig,
    interner: &Interner,
    cli_program: Option<&crate::cli::CliProgram>,
) -> SemanticResult {
    let mut errors = Vec::new();
    let mut types = TypeTable::new();
    let mut resolver = Resolver::new();

    // Establish the AST-level symbol graph before any pass tries to bind names.
    if let Err(e) = passes::collect::collect(program, &mut resolver, &mut types) {
        errors.extend(e);
    }

    // Resolve still runs after collection errors so users see as many name
    // diagnostics as possible before the hard stop at the AST/HIR boundary.
    if let Err(e) = passes::resolve::resolve(program, &mut resolver, interner, &mut types) {
        errors.extend(e);
    }

    // HIR lowering assumes definition identities are coherent; stop before
    // converting syntax when the AST-level contract is already broken.
    if !errors.is_empty() {
        return SemanticResult { hir: None, types, errors };
    }

    let (mut hir, lower_errors) = crate::hir::lower_with_cli(program, &resolver, &mut types, interner, cli_program);

    for err in lower_errors {
        errors.push(SemanticError::new(SemanticErrorKind::LoweringError, err.message, err.span));
    }

    // Typecheck and later analyses assume HIR nodes carry the required semantic
    // attachments; lowering diagnostics are therefore fatal for this pipeline.
    if !errors.is_empty() {
        return SemanticResult { hir: None, types, errors };
    }

    if let Err(e) = passes::typecheck::typecheck_with_interner(&mut hir, &resolver, &mut types, Some(interner)) {
        errors.extend(e);
    }

    if config.borrow_analysis {
        if let Err(e) = passes::borrow::analyze(&hir, &resolver, &types) {
            errors.extend(e);
        }
    }

    if config.exhaustiveness {
        if let Err(e) = passes::exhaustive::check(&hir, &types) {
            errors.extend(e);
        }
    }

    if config.lint {
        if let Err(e) = passes::lint::lint(&hir, &resolver, &types) {
            errors.extend(e);
        }
    }

    SemanticResult { hir: Some(hir), types, errors }
}
