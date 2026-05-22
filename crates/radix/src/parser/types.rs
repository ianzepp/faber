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
//! - Function types: First-class function type syntax (A, B) → C
//! - Array postfix: Array brackets [] applied postfix to base type
//!
//! GRAMMAR COVERAGE
//! ================
//! - Nullable via union: T ∪ nihil (lowers to Option<T> in Phase 3)
//! - Ownership modes: de Type (borrow), in Type (mutable borrow)
//! - Named types: Ident or Ident<Type, Type>
//! - Function types: (Type, Type) → Type
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
use crate::lexer::{Span, TokenKind};
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
        // Check for type start (mode prefix or base: '_', ident, '(', or later ∪ handled inside parse_type)
        if self.check_keyword(TokenKind::De)
            || self.check_keyword(TokenKind::In)
            || matches!(self.peek().kind, TokenKind::Underscore(_))
            || matches!(self.peek().kind, TokenKind::Ident(_))
            || self.check(&TokenKind::LParen)
            || self.check(&TokenKind::Cup)
        {
            Ok(Some(self.parse_type()?))
        } else {
            Ok(None)
        }
    }

    /// Parse a type annotation.
    ///
    /// GRAMMAR (Phase 2):
    ///   type := ['de'|'in'] (func-type | named-type | union-type) ('[]')*
    ///   func-type := '(' type-list ')' '→' type
    ///   named-type := ident ['<' type-list '>']
    ///   union-type := type '∪' type ('∪' type)*
    ///
    /// WHY: Post-Phase 2, nullable optionality is expressed via `T ∪ nihil` union
    /// syntax in pure type positions. Declaration-level optionality uses `sponte`
    /// after the name (see parse_param_list / parse_class_member).
    ///
    /// EDGE: 'nihil' is both a value literal and type name. In type position,
    /// parsed as type name via keyword_ident. Unions are flat lists.
    pub(super) fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let start = self.current_span();

        // Ownership mode: de/in (prefix). Nullable is no longer a prefix; use T ∪ nihil.
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
            let core = TypeExpr { nullable: false, mode, kind: TypeExprKind::Func(func), span };
            return self.parse_union_tail(core, start);
        }

        // Inferred type marker: _
        let mut kind = if matches!(self.peek().kind, TokenKind::Underscore(_)) {
            self.advance();
            TypeExprKind::Infer
        } else {
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

            TypeExprKind::Named(name, params)
        };

        // Array postfix: Type[][]
        while self.check(&TokenKind::LBracket) {
            self.advance();
            self.expect(&TokenKind::RBracket, "expected ']'")?;
            let span = start.merge(self.previous_span());
            kind = TypeExprKind::Array(Box::new(TypeExpr { nullable: false, mode: None, kind, span }));
        }

        let span = start.merge(self.previous_span());
        let core = TypeExpr { nullable: false, mode, kind, span };
        self.parse_union_tail(core, start)
    }

    /// After parsing a core type (named/func/array), consume any trailing `∪ T` chain.
    fn parse_union_tail(&mut self, first: TypeExpr, start: Span) -> Result<TypeExpr, ParseError> {
        if !self.eat(&TokenKind::Cup) {
            return Ok(first);
        }
        let mut members = Vec::new();
        flatten_union_member(first, &mut members);
        // Parse the next member as a full type, then flatten any nested union
        // shape produced by the recursive parse.
        flatten_union_member(self.parse_type()?, &mut members);
        while self.eat(&TokenKind::Cup) {
            flatten_union_member(self.parse_type()?, &mut members);
        }
        let span = start.merge(self.previous_span());
        Ok(TypeExpr {
            nullable: false,
            mode: None, // modes belong on members or parenthesized forms (future)
            kind: TypeExprKind::Union(members),
            span,
        })
    }

    /// Parse function type: (A, B) → C ⇥ E
    ///
    /// GRAMMAR:
    ///   func-type := '(' [type (',' type)*] ')' '→' type ['⇥' type]
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
        self.expect(&TokenKind::Arrow, "expected '→'")?;

        let ret = Box::new(self.parse_type()?);
        let err = if self.eat(&TokenKind::ExitArrow) {
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        Ok(FuncTypeExpr { params, ret, err })
    }
}

fn flatten_union_member(member: TypeExpr, members: &mut Vec<TypeExpr>) {
    if member.nullable || member.mode.is_some() {
        members.push(member);
        return;
    }

    let TypeExpr { nullable, mode, kind, span } = member;
    match kind {
        TypeExprKind::Union(nested) => {
            for nested_member in nested {
                flatten_union_member(nested_member, members);
            }
        }
        kind => members.push(TypeExpr { nullable, mode, kind, span }),
    }
}
