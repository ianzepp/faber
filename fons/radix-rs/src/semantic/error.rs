//! Semantic error types
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Defines error and warning types for all semantic analysis passes. Errors
//! are structured to distinguish categories (name resolution, type checking,
//! borrowing) and include optional help text for common mistakes.
//!
//! COMPILER PHASE: Semantic
//! INPUT: N/A (these are output structures)
//! OUTPUT: Returned by analyze() in SemanticResult
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Categorized Errors: SemanticErrorKind distinguishes error classes,
//!   enabling targeted error handling and filtering
//! - Warnings as Errors: WarningKind is embedded in SemanticErrorKind,
//!   allowing warnings to flow through the same error collection infrastructure
//! - Optional Help: with_help() method provides actionable suggestions without
//!   cluttering the error structure for cases where help isn't available

use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

impl SemanticError {
    pub fn new(kind: SemanticErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self { kind, message: message.into(), span, help: None }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn is_error(&self) -> bool {
        !matches!(self.kind, SemanticErrorKind::Warning(_))
    }
}

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
