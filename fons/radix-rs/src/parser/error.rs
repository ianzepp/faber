//! Parser error types

use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    // Lexer passthrough
    LexError,

    // General
    Expected,
    Unexpected,

    // Declarations
    InvalidDeclaration,
    MissingFunctionName,
    MissingFunctionBody,
    MissingClassName,
    MissingClassBody,
    InvalidParameter,
    InvalidTypeParameter,
    InvalidModifier,
    InvalidAnnotation,
    DuplicateModifier,

    // Statements
    InvalidStatement,
    MissingCondition,
    MissingBlock,
    InvalidPattern,
    InvalidMatchArm,
    InvalidCaseValue,

    // Expressions
    InvalidExpression,
    InvalidLiteral,
    InvalidOperator,
    UnterminatedGroup,
    InvalidCallArgument,
    InvalidMemberAccess,
    InvalidAssignmentTarget,

    // Types
    InvalidType,
    InvalidTypeAnnotation,
    UnterminatedTypeParams,

    // Imports/Directives
    InvalidImport,
    InvalidDirective,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}
