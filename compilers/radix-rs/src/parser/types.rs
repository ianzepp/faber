//! Type Annotation Parsing
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module handles parsing of type annotations used in variable declarations,
//! function signatures, type aliases, and other type-bearing constructs. It supports
//! nullable types, ownership modes, generic type parameters, function types, and arrays.
//!
//! COMPILER PHASE: Parsing
//! INPUT: Token stream (via Parser methods)
//! OUTPUT: TypeExpr AST nodes
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Type-first syntax: Types appear before names in all declarations
//! - Modifier prefixes: Nullable (si), ownership (de/in) modify base types
//! - Generic type parameters: Angle-bracket syntax like TypeScript/Rust
//! - Function types: First-class function type syntax (A, B) -> C
//! - Array postfix: Array brackets [] applied postfix to base type
//!
//! GRAMMAR COVERAGE
//! ================
//! - Nullable: si Type (Option<Type> semantics)
//! - Ownership modes: de Type (borrow), in Type (mutable borrow)
//! - Named types: Ident or Ident<Type, Type>
//! - Function types: (Type, Type) -> Type
//! - Array types: Type[] or Type[][]
//! - Nil type: nihil (void/unit type)
//!
//! TRADE-OFFS
//! ==========
//! - Prefix modifiers only: Nullable and ownership must appear before base type,
//!   cannot mix with postfix array syntax (e.g., no Type[]? in TypeScript sense)
//! - No tuple syntax: Function types handle multiple values, dedicated tuple
//!   syntax deferred to avoid parsing ambiguity

use super::{ParseError, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// TYPE PARSING
// =============================================================================

impl Parser {
    /// Try to parse a type annotation (returns None if not a type).
    ///
    /// WHY: Some contexts need to speculatively check for type annotations
    /// without committing. Used internally for disambiguation.
    #[allow(dead_code)]
    pub(super) fn try_parse_type(&mut self) -> Result<Option<TypeExpr>, ParseError> {
        // Check for type modifiers or type name
        if self.check_keyword(TokenKind::Si)
            || self.check_keyword(TokenKind::De)
            || self.check_keyword(TokenKind::In)
            || matches!(self.peek().kind, TokenKind::Ident(_))
            || self.check(&TokenKind::LParen)
        {
            Ok(Some(self.parse_type()?))
        } else {
            Ok(None)
        }
    }

    /// Parse a type annotation.
    ///
    /// GRAMMAR:
    ///   type := ['si'] ['de'|'in'] (func-type | named-type) ('[]')*
    ///   func-type := '(' type-list ')' '->' type
    ///   named-type := ident ['<' type-list '>']
    ///
    /// WHY: Type expressions combine optional prefix modifiers (nullable, ownership),
    /// a base type (named or function), and optional postfix array dimensions.
    ///
    /// EDGE: 'nihil' is both a value literal and type name. In type position,
    /// parsed as type name via keyword_ident.
    pub(super) fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let start = self.current_span();

        // Nullable modifier: si Type
        let nullable = self.eat_keyword(TokenKind::Si);

        // Ownership mode: de/in
        // WHY: Ownership modes control aliasing and mutation:
        // - de Type: immutable borrow (Rust's &T)
        // - in Type: mutable borrow (Rust's &mut T)
        // - (none): owned value
        let mode = if self.eat_keyword(TokenKind::De) {
            Some(TypeMode::Ref)
        } else if self.eat_keyword(TokenKind::In) {
            Some(TypeMode::MutRef)
        } else {
            None
        };

        // Function type: (params) -> ret
        if self.check(&TokenKind::LParen) {
            let func = self.parse_func_type()?;
            let span = start.merge(self.previous_span());
            return Ok(TypeExpr { nullable, mode, kind: TypeExprKind::Func(func), span });
        }

        // Named type (including 'nihil' as type name)
        let name = if self.check_keyword(TokenKind::Nihil) {
            let span = self.peek().span;
            self.advance();
            self.keyword_ident("nihil", span)
        } else {
            self.parse_ident()?
        };

        // Generic type parameters: Type<A, B>
        let params = if self.eat(&TokenKind::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::Gt, "expected '>'")?;
            params
        } else {
            Vec::new()
        };

        let mut kind = TypeExprKind::Named(name, params);

        // Array postfix: Type[][]
        // WHY: Arrays applied postfix allow natural reading: "Array of Type"
        // becomes "Type[]". Multi-dimensional arrays stack naturally: Type[][]
        while self.check(&TokenKind::LBracket) {
            self.advance();
            self.expect(&TokenKind::RBracket, "expected ']'")?;
            let span = start.merge(self.previous_span());
            kind = TypeExprKind::Array(Box::new(TypeExpr { nullable: false, mode: None, kind, span }));
        }

        let span = start.merge(self.previous_span());
        Ok(TypeExpr { nullable, mode, kind, span })
    }

    /// Parse function type: (A, B) -> C
    ///
    /// GRAMMAR:
    ///   func-type := '(' [type (',' type)*] ')' '->' type
    ///
    /// WHY: First-class function types for callbacks, higher-order functions, and
    /// interface method signatures. Parameter types separated by commas, return
    /// type after arrow.
    fn parse_func_type(&mut self) -> Result<FuncTypeExpr, ParseError> {
        self.expect(&TokenKind::LParen, "expected '('")?;

        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            params.push(self.parse_type()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::RParen, "expected ')'")?;
        self.expect(&TokenKind::Arrow, "expected '->'")?;

        let ret = Box::new(self.parse_type()?);

        Ok(FuncTypeExpr { params, ret })
    }
}
