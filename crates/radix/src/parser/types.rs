//! Type-position grammar for declaration and signature parsing.
//!
//! This module is the parser boundary for syntax that is known to be in type
//! position: variable and field declarations, parameters, aliases, function
//! signatures, casts, and annotation schemas. It does not decide whether a type
//! exists or whether generic arity is valid; it only builds the source shape that
//! later phases resolve.
//!
//! DESIGN PHILOSOPHY
//! =================
//! Faber type syntax is explicit and conservative. Ownership modes are prefix
//! markers (`de`, `in`), arrays are postfix `[]`, function types use arrow
//! syntax, and nullable value slots are expressed by unioning with `nihil`.
//! Optional declaration slots are not type suffixes; they are parsed as
//! post-name `sponte` markers by declaration parsers.
//!
//! INVARIANTS
//! ==========
//! - There is no `Type?` grammar and no nullable prefix shorthand.
//! - `ignotum` is just a named type at this layer, not a nullability marker.
//! - `nihil` may appear as a type member because it is accepted in type position.
//! - Union parsing flattens nested union ASTs when no member-level mode or legacy
//!   nullable flag would be lost.

use super::{ParseError, Parser};
use crate::lexer::{Span, TokenKind};
use crate::syntax::*;

// =============================================================================
// TYPE PARSING
// =============================================================================

impl Parser {
    /// Try to parse a type expression without requiring one.
    ///
    /// This helper is intentionally conservative: it only enters type parsing on
    /// tokens that can begin canonical type syntax. Callers that know they are in
    /// type position should call `parse_type` and receive a diagnostic instead.
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

    /// Parse a required type expression.
    ///
    /// GRAMMAR (Phase 2):
    ///   type := ['de'|'in'] (func-type | named-type | union-type) ('[]')*
    ///   func-type := '(' type-list ')' '→' type
    ///   named-type := ident ['<' type-list '>']
    ///   union-type := type '∪' type ('∪' type)*
    ///
    /// Nullable value types are represented in pure type syntax as `T ∪ nihil`.
    /// Declaration-level optional slots are parsed outside this module as
    /// `sponte` markers after the binding name. `nihil` is accepted here as a
    /// keyword-backed type name because it also exists as a value literal.
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

    /// Consume a trailing union chain after a core type.
    ///
    /// The parser flattens nested union members so later phases do not need to
    /// distinguish `A ∪ (B ∪ C)` from `A ∪ B ∪ C`. Member-level mode/nullable
    /// state is preserved by declining to flatten when it would change meaning.
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

    /// Parse a function type: `(A, B) → C ⇥ E`.
    ///
    /// GRAMMAR:
    ///   func-type := '(' [type (',' type)*] ')' '→' type ['⇥' type]
    ///
    /// Function type syntax is only recognized after an opening parenthesis in a
    /// known type position, avoiding ambiguity with parenthesized expressions.
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
