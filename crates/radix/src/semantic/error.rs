//! Semantic diagnostic taxonomy.
//!
//! Semantic passes share one diagnostic shape so the driver can collect,
//! downgrade, render, and report errors without losing the phase that produced
//! them. The taxonomy is deliberately broad: name binding, HIR lowering,
//! typechecking, ownership analysis, control-flow checks, and lints all report
//! through [`SemanticError`].
//!
//! WARNING POLICY
//! ==============
//! Warnings are represented as [`SemanticErrorKind::Warning`] so they can keep
//! the same source span, message, and optional help text as hard diagnostics.
//! They are not fatal by default: [`SemanticError::is_error`] is the boundary
//! used by `SemanticResult::success` to distinguish warnings from compilation
//! blockers.
//!
//! COMPATIBILITY
//! =============
//! Some front-end check modes intentionally downgrade unresolved-name failures
//! for permissive exploratory flows. [`SemanticErrorKind::is_permissive_check_downgrade`]
//! names that policy in one place instead of scattering ad hoc kind checks
//! through command surfaces.

use crate::lexer::Span;

/// One semantic diagnostic with source position and optional corrective help.
///
/// The type is used for both hard errors and warnings; call [`Self::is_error`]
/// when a caller needs the fatal/non-fatal distinction.
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

impl SemanticError {
    /// Build a diagnostic without extra help text.
    pub fn new(kind: SemanticErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self { kind, message: message.into(), span, help: None }
    }

    /// Attach a short user-facing suggestion while preserving the diagnostic kind.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Return whether this diagnostic should block successful compilation.
    pub fn is_error(&self) -> bool {
        !matches!(self.kind, SemanticErrorKind::Warning(_))
    }
}

/// Semantic diagnostic categories used by passes and command policy.
///
/// Variants are intentionally phase-oriented rather than renderer-oriented:
/// renderers should use the message/help text for display, while compiler
/// policy can match the kind when it needs to distinguish unresolved names,
/// type failures, ownership failures, or warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticErrorKind {
    // Name resolution
    UndefinedVariable,
    UndefinedType,
    UndefinedFunction,
    UndefinedMember,
    DuplicateDefinition,
    ImportNotFound,
    CircularDependency,
    LoweringError,
    CliValidation,
    ShadowedVariable,

    // Type errors
    TypeMismatch,
    InvalidOperandTypes,
    NotCallable,
    WrongArity,
    MissingTypeAnnotation,
    InvalidCast,
    InvalidConversion,

    // Assignment
    ImmutableAssignment,
    InvalidAssignmentTarget,

    // Control flow
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    MissingReturn,

    // Pattern matching
    NonExhaustiveMatch,
    UnreachablePattern,
    DuplicatePattern,

    // Borrowing (for Rust target)
    UseAfterMove,
    BorrowOfMoved,
    MutableBorrowConflict,
    CannotMoveOut,
    LifetimeMismatch,
    AssignToImmutableBorrow,
    ModeMismatch,

    // Warnings
    Warning(WarningKind),
}

impl SemanticErrorKind {
    /// Return whether permissive check mode may report this hard error as a softer result.
    ///
    /// The downgrade set is restricted to missing symbols/imports because those
    /// can arise while users explore incomplete files. Type, control-flow, and
    /// lowering failures still indicate that the current program shape is known
    /// to be invalid.
    pub fn is_permissive_check_downgrade(self) -> bool {
        matches!(
            self,
            SemanticErrorKind::UndefinedVariable
                | SemanticErrorKind::UndefinedType
                | SemanticErrorKind::UndefinedFunction
                | SemanticErrorKind::UndefinedMember
                | SemanticErrorKind::ImportNotFound
        )
    }
}

/// Non-fatal semantic findings that share the normal diagnostic transport.
///
/// These variants communicate quality, portability, or intent issues without
/// causing [`SemanticResult`](super::SemanticResult) to fail unless a caller
/// layers on a stricter warnings-as-errors policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    UnusedVariable,
    UnusedImport,
    UnusedFunction,
    UnreachableCode,
    UnnecessaryCast,
    DeprecatedFeature,
    TargetNoop,
    UnusedMutRefParam,
    UnusedMoveParam,
    ExplicitIgnotumAnnotation,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SemanticError {}
