//! Semantic error types

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
        Self {
            kind,
            message: message.into(),
            span,
            help: None,
        }
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

    // Warnings
    Warning(WarningKind),
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
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SemanticError {}
