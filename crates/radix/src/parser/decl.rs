//! Declaration grammar for Faber's crate-facing syntax tree.
//!
//! This module owns the parser surface for declarations and declaration-adjacent
//! blocks: bindings, functions, class/interface shapes, algebraic data types,
//! imports, directives, tests, annotations, and ordinary block bodies. It is the
//! main grammar boundary between source-level declarations and AST contracts
//! consumed by the collector and package CLI.
//!
//! DESIGN PHILOSOPHY
//! =================
//! Declarations are intentionally explicit: types precede names, inferred locals
//! require `_`, voluntary slots use post-name `sponte`, and canonical modifier
//! ordering is enforced in the parser when later phases would otherwise receive
//! ambiguous ASTs. Where legacy or transitional syntax is still recognized, the
//! parser either normalizes it into the current AST contract or emits a targeted
//! diagnostic instead of expanding the language silently.
//!
//! INVARIANTS
//! ==========
//! - `parse_statement` is the dispatcher for both top-level and nested statement
//!   contexts; recovery boundaries in `mod.rs` must stay aligned with it.
//! - `§` directives are file-scope only and are parsed before ordinary statements.
//! - Function, parameter, field, and closure signatures delegate to `types.rs`
//!   for type syntax; this file decides where type positions are legal.
//! - Annotation parsers define the source contract for CLI/package metadata,
//!   while unknown annotations remain statement annotations for later phases.

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// STATEMENT DISPATCH
// =============================================================================

impl Parser {
    /// Dispatch a top-level or nested statement by its leading token.
    ///
    /// GRAMMAR:
    ///   statement := declaration | control-flow | transfer | block | expr-stmt
    ///
    /// This function is part of the parser's recovery contract. Any keyword added
    /// here as a statement start should also be considered for
    /// `Parser::is_recovery_boundary` so malformed input can resume at the new
    /// construct instead of skipping it.
    pub(super) fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let id = self.next_id();

        let kind = match self.peek().kind {
            // Declarations
            TokenKind::Fixum | TokenKind::Varia => self.parse_var_decl()?,
            TokenKind::Functio => self.parse_func_decl()?,
            TokenKind::Genus => self.parse_class_decl()?,
            TokenKind::Pactum => self.parse_interface_decl()?,
            TokenKind::Typus => self.parse_type_alias_decl()?,
            TokenKind::Ordo => self.parse_enum_decl()?,
            TokenKind::Discretio => self.parse_union_decl()?,
            TokenKind::Importa => self.parse_import_decl()?,
            TokenKind::Section => {
                return Err(self.error(ParseErrorKind::InvalidDirective, "directives must appear at file scope"));
            }
            TokenKind::Ex => self.parse_extract_stmt()?,
            TokenKind::Probandum => self.parse_probandum_decl()?,
            TokenKind::Proba => StmtKind::Proba(self.parse_proba_case()?),
            TokenKind::Abstractus => self.parse_abstract_class_decl()?,

            // Control flow
            TokenKind::Si => self.parse_si_stmt()?,
            TokenKind::Dum => self.parse_dum_stmt()?,
            TokenKind::Itera => self.parse_itera_stmt()?,
            TokenKind::Elige => self.parse_elige_stmt()?,
            TokenKind::Discerne => self.parse_discerne_stmt()?,
            TokenKind::Custodi => self.parse_custodi_stmt()?,
            TokenKind::Fac => self.parse_fac_stmt()?,

            // Transfer
            TokenKind::Redde => self.parse_redde_stmt()?,
            TokenKind::Rumpe => self.parse_rumpe_stmt()?,
            TokenKind::Perge => self.parse_perge_stmt()?,
            TokenKind::Iace => self.parse_iace_stmt()?,
            TokenKind::Mori => self.parse_mori_stmt()?,
            TokenKind::Tacet => self.parse_tacet_stmt()?,

            // Error handling
            TokenKind::Tempta => self.parse_tempta_stmt()?,
            TokenKind::Adfirma => self.parse_adfirma_stmt()?,

            // Output
            TokenKind::Scribe | TokenKind::Vide | TokenKind::Mone | TokenKind::Nota => self.parse_scribe_stmt()?,

            // Entry points
            TokenKind::Incipit | TokenKind::Incipiet => self.parse_incipit_stmt()?,

            // Resource management
            TokenKind::Cura => self.parse_cura_stmt()?,

            // Endpoint
            TokenKind::Ad => self.parse_ad_stmt()?,

            // Block
            TokenKind::LBrace => StmtKind::Block(self.parse_block()?),

            // Annotations (for following statement)
            TokenKind::At => {
                let annotations = self.parse_annotations()?;
                let mut stmt = self.parse_statement()?;
                stmt.annotations = annotations;
                return Ok(stmt);
            }

            // Expression statement
            _ => self.parse_expr_stmt()?,
        };

        let span = start.merge(self.previous_span());
        Ok(Stmt { id, kind, span, annotations: Vec::new() })
    }

    // =============================================================================
    // VARIABLE DECLARATIONS
    // =============================================================================

    /// Parse `fixum` and `varia` binding declarations.
    ///
    /// GRAMMAR:
    ///   var-decl := ('fixum' | 'varia') type pattern ['←' expr]
    ///
    /// Simple bindings carry an explicit type slot, including `_` for inference.
    /// Destructuring keeps its historical untyped shape because the binding
    /// pattern itself owns the names and any rest element.
    fn parse_var_decl(&mut self) -> Result<StmtKind, ParseError> {
        let (mutability, is_await) = match self.peek().kind {
            TokenKind::Fixum => {
                self.advance();
                (Mutability::Immutable, false)
            }
            TokenKind::Varia => {
                self.advance();
                (Mutability::Mutable, false)
            }
            _ => unreachable!(),
        };

        // Destructuring keeps the historical untyped pattern form; simple bindings
        // use an explicit type slot, with `_` meaning "infer from the initializer".
        let ty = if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
            None
        } else {
            Some(self.parse_type()?)
        };

        // Binding pattern
        let binding = self.parse_binding_pattern()?;

        // Optional initializer
        let init = if self.eat(&TokenKind::Assign) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Var(VarDecl { mutability, is_await, ty, binding, init }))
    }

    fn parse_binding_pattern(&mut self) -> Result<BindingPattern, ParseError> {
        if let TokenKind::LBracket = self.peek().kind {
            return self.parse_array_binding_pattern();
        }

        if let TokenKind::LBrace = self.peek().kind {
            return self.parse_object_binding_pattern();
        }

        if let TokenKind::Underscore(_) = self.peek().kind {
            let span = self.peek().span;
            self.advance();
            return Ok(BindingPattern::Wildcard(span));
        }

        let ident = self.parse_ident()?;
        Ok(BindingPattern::Ident(ident))
    }

    fn parse_array_binding_pattern(&mut self) -> Result<BindingPattern, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBracket, "expected '['")?;

        let mut elements = Vec::new();
        let mut rest = None;

        while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Ceteri) {
                if rest.is_some() {
                    return Err(self.error(ParseErrorKind::Expected, "rest pattern already specified"));
                }
                let name = self.parse_ident()?;
                rest = Some(name);
                break;
            }

            let element = if let TokenKind::Underscore(_) = self.peek().kind {
                let span = self.peek().span;
                self.advance();
                BindingPattern::Wildcard(span)
            } else if let TokenKind::LBracket = self.peek().kind {
                self.parse_array_binding_pattern()?
            } else {
                BindingPattern::Ident(self.parse_ident()?)
            };

            elements.push(element);

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::RBracket, "expected ']' after pattern")?;
        let span = start.merge(self.previous_span());
        Ok(BindingPattern::Array { elements, rest, span })
    }

    fn parse_object_binding_pattern(&mut self) -> Result<BindingPattern, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut fields = Vec::new();
        let mut rest = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Ceteri) {
                if rest.is_some() {
                    return Err(self.error(ParseErrorKind::Expected, "rest pattern already specified"));
                }
                rest = Some(self.parse_ident()?);
                break;
            }

            let name = self.parse_ident()?;
            let alias = if self.eat_keyword(TokenKind::Ut) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            fields.push(ExField { name, alias });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}' after pattern")?;
        let span = start.merge(self.previous_span());
        Ok(BindingPattern::Object { fields, rest, span })
    }

    // =============================================================================
    // FUNCTION DECLARATIONS
    // =============================================================================

    /// Parse function declaration.
    ///
    /// GRAMMAR:
    ///   func-decl := 'functio' ident '(' params ')' modifiers ['→' type] [block]
    ///   params    := type-param* regular-param*
    ///   type-param := 'prae' 'typus' ident
    ///
    /// WHY: Functions can have type parameters (for generics), ownership mode
    /// parameters (de/in/ex), and trailing modifiers (curata, errata, etc.).
    fn parse_func_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Functio, "expected 'functio'")?;

        let name = self.parse_ident()?;

        self.expect(&TokenKind::LParen, "expected '(' after function name")?;
        let (type_params, params) = self.parse_param_list()?;
        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;

        let modifiers = self.parse_func_modifiers()?;

        let ret = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let err = if self.eat(&TokenKind::ExitArrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.check(&TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(StmtKind::Func(FuncDecl {
            name,
            type_params,
            params,
            modifiers,
            ret,
            err,
            body,
            annotations: Vec::new(),
        }))
    }

    /// Parse type parameters followed by regular parameters.
    ///
    /// GRAMMAR:
    ///   param-list := type-param* (',' type-param)* regular-param* (',' regular-param)*
    ///   type-param := 'prae' 'typus' ident
    ///   regular-param := ['de'|'in'|'ex'] ['ceteri'] type ident [sponte] [fixus] ['ut' ident] ['vel' expr]
    ///
    /// Parameter order is part of the language contract: `prae typus` parameters
    /// must precede value parameters, and value parameters remain type-first.
    /// `sponte` and `fixus` are post-name markers so nullability and fixedness do
    /// not become suffix syntax on the type expression.
    fn parse_param_list(&mut self) -> Result<(Vec<TypeParam>, Vec<Param>), ParseError> {
        let mut type_params = Vec::new();
        let mut params = Vec::new();

        // Type parameters first (prae typus T)
        while self.check_keyword(TokenKind::Prae) {
            self.advance();
            self.expect_keyword(TokenKind::Typus, "expected 'typus' after 'prae'")?;
            let name = self.parse_ident()?;
            type_params.push(TypeParam { span: name.span, name });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        // Regular parameters
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let start = self.current_span();

            // Mode: de/in/ex (prefix on type)
            let mode = if self.eat_keyword(TokenKind::De) {
                ParamMode::Ref
            } else if self.eat_keyword(TokenKind::In) {
                ParamMode::MutRef
            } else if self.eat_keyword(TokenKind::Ex) {
                ParamMode::Move
            } else {
                ParamMode::Owned
            };

            // Rest: ceteri
            let rest = self.eat_keyword(TokenKind::Ceteri);

            // Type
            let ty = self.parse_type()?;

            // Name
            let name = self.parse_ident()?;

            // Post-name markers: sponte (voluntary), fixus (fixed-after-init). Canonical order only.
            let sponte = self.eat_keyword(TokenKind::Sponte);
            let fixus = self.eat_keyword(TokenKind::Fixus);
            if !sponte && self.check_keyword(TokenKind::Sponte) {
                return Err(self.error(
                    ParseErrorKind::InvalidParameter,
                    "unexpected 'sponte' after 'fixus'; canonical order is '<type> <name> [sponte] [fixus] [vel default]'",
                ));
            }

            // Alias: ut NAME
            let alias = if self.eat_keyword(TokenKind::Ut) {
                Some(self.parse_ident()?)
            } else {
                None
            };

            // Default: vel EXPR
            let default = if self.eat_keyword(TokenKind::Vel) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            let span = start.merge(self.previous_span());
            params.push(Param { sponte, fixus, mode, rest, ty, name, alias, default, span });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok((type_params, params))
    }

    /// Parse trailing function modifiers that affect call/runtime contracts.
    fn parse_func_modifiers(&mut self) -> Result<Vec<FuncModifier>, ParseError> {
        let mut modifiers = Vec::new();

        loop {
            if self.eat_keyword(TokenKind::Argumenta) {
                let name = self.parse_ident()?;
                modifiers.push(FuncModifier::Argumenta(name));
            } else if self.eat_keyword(TokenKind::Curata) {
                let required = self.parse_ident()?;
                let alias = if self.eat_keyword(TokenKind::Ut) {
                    Some(self.parse_ident()?)
                } else {
                    None
                };
                modifiers.push(FuncModifier::Curata { required, alias });
            } else if self.eat_keyword(TokenKind::Errata) {
                let name = self.parse_ident()?;
                modifiers.push(FuncModifier::Errata(name));
            } else if self.eat_keyword(TokenKind::Exitus) {
                let value = if let TokenKind::Integer(n) = self.peek().kind {
                    self.advance();
                    ExitusValue::Number(n)
                } else {
                    ExitusValue::Name(self.parse_ident()?)
                };
                modifiers.push(FuncModifier::Exitus(value));
            } else if self.eat_keyword(TokenKind::Immutata) {
                modifiers.push(FuncModifier::Immutata);
            } else if self.eat_keyword(TokenKind::Iacit) {
                modifiers.push(FuncModifier::Iacit);
            } else if self.eat_keyword(TokenKind::Optiones) {
                let name = self.parse_ident()?;
                modifiers.push(FuncModifier::Optiones(name));
            } else {
                break;
            }
        }

        Ok(modifiers)
    }

    // =============================================================================
    // CLASS DECLARATIONS
    // =============================================================================

    /// Parse a concrete class declaration.
    ///
    /// GRAMMAR:
    ///   class-decl := 'genus' ident ['<' type-params '>'] ['sub' ident] ['implet' ident-list] '{' member* '}'
    ///
    /// Class headers are intentionally narrow: one optional `sub` parent and a
    /// comma-separated `implet` list. Field and method parsing stays inside this
    /// module because member syntax shares declaration markers with top-level
    /// bindings and functions.
    fn parse_class_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.parse_class_decl_inner(false)
    }

    /// Parse `abstractus genus ...` without introducing a second class grammar.
    fn parse_abstract_class_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Abstractus, "expected 'abstractus'")?;
        self.parse_class_decl_inner(true)
    }

    fn parse_class_decl_inner(&mut self, is_abstract: bool) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Genus, "expected 'genus'")?;

        let name = self.parse_ident()?;
        let type_params = self.try_parse_type_params()?;

        // Extends
        let extends = if self.eat_keyword(TokenKind::Sub) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        // Implements
        let mut implements = Vec::new();
        if self.eat_keyword(TokenKind::Implet) {
            loop {
                implements.push(self.parse_ident()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::LBrace, "expected '{' after class header")?;

        let mut members = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let member = self.parse_class_member()?;
            members.push(member);
        }

        self.expect(&TokenKind::RBrace, "expected '}' after class body")?;

        Ok(StmtKind::Class(ClassDecl {
            is_abstract,
            name,
            type_params,
            extends,
            implements,
            members,
        }))
    }

    /// Parse a class member, preserving annotations at the member boundary.
    fn parse_class_member(&mut self) -> Result<ClassMember, ParseError> {
        let start = self.current_span();

        let annotations = self.parse_annotations()?;

        let kind = if self.check_keyword(TokenKind::Functio) {
            // Method
            if let StmtKind::Func(func) = self.parse_func_decl()? {
                ClassMemberKind::Method(func)
            } else {
                unreachable!()
            }
        } else {
            // Field
            let is_static = self.eat_keyword(TokenKind::Generis);
            let is_bound = self.eat_keyword(TokenKind::Nexum);

            let ty = self.parse_type()?;
            let name = self.parse_ident()?;

            // Post-name declaration markers for fields (mirrors param syntax)
            let sponte = self.eat_keyword(TokenKind::Sponte);
            let fixus = self.eat_keyword(TokenKind::Fixus);
            if !sponte && self.check_keyword(TokenKind::Sponte) {
                return Err(self.error(
                    ParseErrorKind::InvalidParameter,
                    "unexpected 'sponte' after 'fixus'; use 'type name sponte fixus' order",
                ));
            }

            let init = if self.eat(&TokenKind::Eq) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            ClassMemberKind::Field(FieldDecl { is_static, is_bound, sponte, fixus, ty, name, init })
        };

        let span = start.merge(self.previous_span());
        Ok(ClassMember { annotations, kind, span })
    }

    // =============================================================================
    // INTERFACE DECLARATIONS
    // =============================================================================

    /// Parse an interface declaration and its method signatures.
    ///
    /// GRAMMAR:
    ///   interface-decl := 'pactum' ident ['<' type-params '>'] '{' method-sig* '}'
    ///   method-sig := annotation* 'functio' ident '(' params ')' modifiers ['→' type]
    ///
    /// Interface methods deliberately reuse function parameter and modifier
    /// parsing, but they stop before bodies so the AST carries a contract rather
    /// than an implementation placeholder.
    fn parse_interface_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Pactum, "expected 'pactum'")?;

        let name = self.parse_ident()?;
        let type_params = self.try_parse_type_params()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            // Parse (and currently discard) annotations on interface methods (e.g. @ externa).
            // WHY (Phase 1): allows stdlib pactum files with annotated methods to parse.
            // Full preservation into AST/HIR + Clone hygiene deferred; annotations on methods
            // carry linking meaning (externa, futura) but are not yet represented on InterfaceMethod.
            if self.check(&TokenKind::At) {
                self.parse_annotations()?;
            }
            let start = self.current_span();

            self.expect_keyword(TokenKind::Functio, "expected 'functio'")?;
            let method_name = self.parse_member_ident()?;

            self.expect(&TokenKind::LParen, "expected '('")?;
            let (_, params) = self.parse_param_list()?;
            self.expect(&TokenKind::RParen, "expected ')'")?;

            let modifiers = self.parse_func_modifiers()?;

            let ret = if self.eat(&TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let err = if self.eat(&TokenKind::ExitArrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let span = start.merge(self.previous_span());
            methods.push(InterfaceMethod { name: method_name, params, modifiers, ret, err, span });
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Interface(InterfaceDecl { name, type_params, methods }))
    }

    /// Parse a type alias declaration.
    fn parse_type_alias_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Typus, "expected 'typus'")?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::Eq, "expected '='")?;
        let ty = self.parse_type()?;

        Ok(StmtKind::TypeAlias(TypeAliasDecl { name, ty }))
    }

    // =============================================================================
    // ENUM AND UNION DECLARATIONS
    // =============================================================================

    /// Parse enum declaration.
    ///
    /// GRAMMAR:
    ///   enum-decl := 'ordo' ident '{' enum-member* '}'
    ///   enum-member := ident ['=' (integer | string)]
    ///
    /// WHY: Enums can have explicit integer or string values for FFI compatibility.
    fn parse_enum_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Ordo, "expected 'ordo'")?;
        let name = self.parse_ident()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut members = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let start = self.current_span();
            let member_name = self.parse_ident()?;

            let value = if self.eat(&TokenKind::Eq) {
                match self.peek().kind {
                    TokenKind::Integer(n) => {
                        self.advance();
                        Some(EnumValue::Integer(n))
                    }
                    TokenKind::Minus => {
                        self.advance();
                        if let TokenKind::Integer(n) = self.peek().kind {
                            self.advance();
                            Some(EnumValue::Integer(-n))
                        } else {
                            return Err(self.error(ParseErrorKind::Expected, "expected number"));
                        }
                    }
                    TokenKind::String(s) => {
                        self.advance();
                        Some(EnumValue::String(s))
                    }
                    _ => return Err(self.error(ParseErrorKind::Expected, "expected value")),
                }
            } else {
                None
            };

            let span = start.merge(self.previous_span());
            members.push(EnumMember { name: member_name, value, span });

            // Optional comma
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Enum(EnumDecl { name, members }))
    }

    /// Parse tagged union declaration.
    ///
    /// GRAMMAR:
    ///   union-decl := 'discretio' ident ['<' type-params '>'] '{' variant* '}'
    ///   variant := ident ['{' field* '}']
    ///   field := type ident
    ///
    /// WHY: Tagged unions (discriminated unions) enable sum types with optional
    /// variant-specific payload fields.
    fn parse_union_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Discretio, "expected 'discretio'")?;
        let name = self.parse_ident()?;
        let type_params = self.try_parse_type_params()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let start = self.current_span();
            let variant_name = self.parse_ident()?;

            let fields = if self.eat(&TokenKind::LBrace) {
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let field_start = self.current_span();
                    let ty = self.parse_type()?;
                    let field_name = self.parse_ident()?;
                    let field_span = field_start.merge(self.previous_span());
                    fields.push(VariantField { ty, name: field_name, span: field_span });
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "expected '}'")?;
                fields
            } else {
                Vec::new()
            };

            let span = start.merge(self.previous_span());
            variants.push(Variant { name: variant_name, fields, span });

            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Union(UnionDecl { name, type_params, variants }))
    }

    // =============================================================================
    // IMPORT AND DIRECTIVE DECLARATIONS
    // =============================================================================

    /// Parse a source import declaration.
    ///
    /// GRAMMAR:
    ///   import-decl := 'importa' 'ex' string ['privata'|'publica'] import-kind
    ///   import-kind := ident ['ut' ident] | '*' 'ut' ident
    ///
    /// The parser only records the import specifier, visibility, and binding
    /// shape. Package and library resolution decide later whether the string
    /// names a local module, stdlib interface, or invalid provider path.
    fn parse_import_decl(&mut self) -> Result<StmtKind, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Importa, "expected 'importa'")?;
        self.expect_keyword(TokenKind::Ex, "expected 'ex'")?;

        let path = self.parse_string()?;

        let visibility = if self.eat_keyword(TokenKind::Privata) {
            Visibility::Private
        } else if self.eat_keyword(TokenKind::Publica) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        let kind = if self.eat(&TokenKind::Star) {
            self.expect_keyword(TokenKind::Ut, "expected 'ut'")?;
            let alias = self.parse_ident()?;
            ImportKind::Wildcard { alias }
        } else {
            let name = self.parse_ident()?;
            let alias = if self.eat_keyword(TokenKind::Ut) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            ImportKind::Named { name, alias }
        };

        let span = start.merge(self.previous_span());
        Ok(StmtKind::Import(ImportDecl { path, visibility, kind, span }))
    }

    /// Parse a file-scope compiler directive.
    ///
    /// GRAMMAR:
    ///   directive := '§' ident arg*
    ///   arg := string | ident
    ///
    /// Directives are intentionally shallow here: the parser records the directive
    /// name and string/identifier arguments, while semantic validation of names
    /// and arity belongs to the phase that consumes each directive.
    pub(super) fn parse_directive_decl(&mut self) -> Result<DirectiveDecl, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::Section, "expected '§'")?;

        let name = self.parse_ident()?;

        let mut args = Vec::new();
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::String(s) => {
                    self.advance();
                    args.push(DirectiveArg::String(s));
                }
                TokenKind::Ident(_) => {
                    let ident = self.parse_ident()?;
                    args.push(DirectiveArg::Ident(ident));
                }
                _ => break,
            }
        }

        let span = start.merge(self.previous_span());
        Ok(DirectiveDecl { name, args, span })
    }

    // =============================================================================
    // TEST DECLARATIONS
    // =============================================================================

    /// Parse test suite declaration.
    ///
    /// GRAMMAR:
    ///   probandum := 'probandum' string proba-modifier* '{' probandum-body '}'
    ///   probandum-body := (setup | proba | nested-probandum)*
    ///
    /// WHY: Test suites support setup/teardown hooks (praepara/postpara), individual
    /// test cases (proba), and nesting for organization.
    fn parse_probandum_decl(&mut self) -> Result<StmtKind, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Probandum, "expected 'probandum'")?;

        let name = self.parse_string()?;
        let modifiers = self.parse_test_modifiers()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let body = self.parse_probandum_body()?;

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let span = start.merge(self.previous_span());
        Ok(StmtKind::Probandum(ProbandumDecl { name, modifiers, body, span }))
    }

    fn parse_probandum_body(&mut self) -> Result<ProbandumBody, ParseError> {
        let mut setup = Vec::new();
        let mut tests = Vec::new();
        let mut nested = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.check_keyword(TokenKind::Praepara)
                || self.check_keyword(TokenKind::Praeparabit)
                || self.check_keyword(TokenKind::Postpara)
                || self.check_keyword(TokenKind::Postparabit)
            {
                let start = self.current_span();
                let kind = match self.peek().kind {
                    TokenKind::Praepara => {
                        self.advance();
                        PraeparaKind::Praepara
                    }
                    TokenKind::Praeparabit => {
                        self.advance();
                        PraeparaKind::Praeparabit
                    }
                    TokenKind::Postpara => {
                        self.advance();
                        PraeparaKind::Postpara
                    }
                    TokenKind::Postparabit => {
                        self.advance();
                        PraeparaKind::Postparabit
                    }
                    _ => unreachable!(),
                };
                let all = self.eat_keyword(TokenKind::Omnia);
                let body = self.parse_block()?;
                let span = start.merge(self.previous_span());
                setup.push(PraeparaBlock { kind, all, body, span });
            } else if self.check_keyword(TokenKind::Probandum) {
                if let StmtKind::Probandum(test) = self.parse_probandum_decl()? {
                    nested.push(test);
                }
            } else if self.check_keyword(TokenKind::Proba) {
                let case = self.parse_proba_case()?;
                tests.push(case);
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected test case or setup"));
            }
        }

        Ok(ProbandumBody { setup, tests, nested })
    }

    fn parse_proba_case(&mut self) -> Result<ProbaCase, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Proba, "expected 'proba'")?;

        let name = self.parse_string()?;

        let modifiers = self.parse_test_modifiers()?;

        let body = self.parse_block()?;

        let span = start.merge(self.previous_span());
        Ok(ProbaCase { modifiers, name, body, span })
    }

    fn parse_test_modifiers(&mut self) -> Result<Vec<ProbaModifier>, ParseError> {
        let mut modifiers = Vec::new();
        loop {
            if self.eat_keyword(TokenKind::Omitte) {
                let reason = self.parse_string()?;
                modifiers.push(ProbaModifier::Omitte(reason));
            } else if self.eat_keyword(TokenKind::Futurum) {
                let reason = self.parse_string()?;
                modifiers.push(ProbaModifier::Futurum(reason));
            } else if self.eat_keyword(TokenKind::Solum) {
                modifiers.push(ProbaModifier::Solum);
            } else if self.eat_keyword(TokenKind::Tag) {
                let tag = self.parse_string()?;
                modifiers.push(ProbaModifier::Tag(tag));
            } else if self.eat_keyword(TokenKind::Temporis) {
                let n = self.parse_test_modifier_integer("expected integer after 'temporis'")?;
                modifiers.push(ProbaModifier::Temporis(n));
            } else if self.eat_keyword(TokenKind::Metior) {
                modifiers.push(ProbaModifier::Metior);
            } else if self.eat_keyword(TokenKind::Repete) {
                let n = self.parse_test_modifier_integer("expected integer after 'repete'")?;
                modifiers.push(ProbaModifier::Repete(n));
            } else if self.eat_keyword(TokenKind::Fragilis) {
                let n = self.parse_test_modifier_integer("expected integer after 'fragilis'")?;
                modifiers.push(ProbaModifier::Fragilis(n));
            } else if self.eat_keyword(TokenKind::Requirit) {
                let req = self.parse_string()?;
                modifiers.push(ProbaModifier::Requirit(req));
            } else if self.eat_keyword(TokenKind::SolumIn) {
                let env = self.parse_string()?;
                modifiers.push(ProbaModifier::SolumIn(env));
            } else {
                break;
            }
        }
        Ok(modifiers)
    }

    fn parse_test_modifier_integer(&mut self, msg: &str) -> Result<i64, ParseError> {
        match self.peek().kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(n)
            }
            _ => Err(self.error(ParseErrorKind::Expected, msg)),
        }
    }

    /// Parse one or more declaration annotations.
    ///
    /// Known CLI/package annotations are parsed into structured AST variants so
    /// `faber` can build command metadata without reparsing token payloads.
    /// Unknown annotations remain token-backed statement annotations for later
    /// target- or stdlib-specific consumers.
    fn parse_annotations(&mut self) -> Result<Vec<Annotation>, ParseError> {
        let mut annotations = Vec::new();

        while self.check(&TokenKind::At) {
            let start = self.current_span();
            self.advance(); // @

            let kind = self.parse_annotation_kind()?;
            let span = start.merge(self.previous_span());

            annotations.push(Annotation { kind, span });
        }

        Ok(annotations)
    }

    fn parse_annotation_kind(&mut self) -> Result<AnnotationKind, ParseError> {
        let name = self.parse_annotation_name()?;
        let annotation_name = self.interner.resolve(name.name).to_owned();

        match annotation_name.as_str() {
            "cli" => return self.parse_cli_annotation(),
            "imperium" => return self.parse_imperium_annotation(),
            "optio" => return self.parse_optio_annotation(),
            "operandus" => return self.parse_operandus_annotation(),
            _ => {}
        }

        let mut args = Vec::new();

        while self.is_annotation_arg() {
            args.push(self.advance().clone());
        }

        Ok(AnnotationKind::Statement(AnnotationStmt { name, args }))
    }

    fn parse_cli_annotation(&mut self) -> Result<AnnotationKind, ParseError> {
        let name = self.parse_string()?;
        self.expect_annotation_end("unexpected token after @ cli name")?;
        Ok(AnnotationKind::Cli(CliAnnotation { name }))
    }

    fn parse_imperium_annotation(&mut self) -> Result<AnnotationKind, ParseError> {
        let name = self.parse_string()?;
        self.expect_annotation_end("unexpected token after @ imperium name")?;
        Ok(AnnotationKind::Imperium(ImperiumAnnotation { name }))
    }

    fn parse_optio_annotation(&mut self) -> Result<AnnotationKind, ParseError> {
        let binding = self.parse_ident()?;
        let mut ty = None;
        let mut short = None;
        let mut long = None;
        let mut description = None;
        let mut global = false;
        let mut default = None;

        while self.is_annotation_arg() {
            if self.eat_annotation_ident("brevis") {
                short = Some(self.parse_string()?);
            } else if self.eat_annotation_ident("longum") {
                long = Some(self.parse_string()?);
            } else if self.eat_annotation_ident("typus") {
                ty = Some(self.parse_type()?);
            } else if self.eat_annotation_ident("descriptio") {
                description = Some(self.parse_string()?);
            } else if self.eat_annotation_ident("ubique") {
                global = true;
            } else if self.eat_annotation_ident("vel") {
                default = Some(Box::new(self.parse_annotation_default()?));
            } else {
                return Err(self.error(ParseErrorKind::InvalidAnnotation, "invalid @ optio modifier"));
            }
        }

        let flag = ty
            .as_ref()
            .is_some_and(|ty| self.is_annotation_bivalens_type(ty));

        Ok(AnnotationKind::Optio(OptioAnnotation {
            binding,
            ty,
            short,
            long,
            flag,
            description,
            global,
            default,
        }))
    }

    fn parse_operandus_annotation(&mut self) -> Result<AnnotationKind, ParseError> {
        let rest = self.eat_annotation_ident("ceteri");
        let ty = self.parse_type()?;
        let binding = self.parse_ident()?;
        let mut description = None;
        let mut global = false;
        let mut default = None;

        while self.is_annotation_arg() {
            if self.eat_annotation_ident("descriptio") {
                description = Some(self.parse_string()?);
            } else if self.eat_annotation_ident("ubique") {
                global = true;
            } else if self.eat_annotation_ident("vel") {
                default = Some(Box::new(self.parse_annotation_default()?));
            } else {
                return Err(self.error(ParseErrorKind::InvalidAnnotation, "invalid @ operandus modifier"));
            }
        }

        Ok(AnnotationKind::Operandus(OperandusAnnotation {
            rest,
            ty,
            binding,
            description,
            global,
            default,
        }))
    }

    fn parse_annotation_name(&mut self) -> Result<Ident, ParseError> {
        let token = self.advance();
        let span = token.span;
        match token.kind {
            TokenKind::Ident(sym) | TokenKind::Underscore(sym) => Ok(Ident { name: sym, span }),
            TokenKind::Publica => Ok(self.keyword_ident("publica", span)),
            TokenKind::Protecta => Ok(self.keyword_ident("protecta", span)),
            TokenKind::Privata => Ok(self.keyword_ident("privata", span)),
            TokenKind::Futura => Ok(self.keyword_ident("futura", span)),
            TokenKind::Cursor => Ok(self.keyword_ident("cursor", span)),
            TokenKind::Tag => Ok(self.keyword_ident("tag", span)),
            TokenKind::Solum => Ok(self.keyword_ident("solum", span)),
            TokenKind::Omitte => Ok(self.keyword_ident("omitte", span)),
            TokenKind::Metior => Ok(self.keyword_ident("metior", span)),
            _ => {
                Err(ParseError { kind: ParseErrorKind::Expected, message: "expected annotation name".to_owned(), span })
            }
        }
    }

    fn eat_annotation_ident(&mut self, expected: &str) -> bool {
        if let TokenKind::Ident(sym) = self.peek().kind {
            if self.interner.resolve(sym) == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect_annotation_end(&mut self, message: &str) -> Result<(), ParseError> {
        if self.is_annotation_arg() {
            Err(self.error(ParseErrorKind::InvalidAnnotation, message))
        } else {
            Ok(())
        }
    }

    fn parse_annotation_default(&mut self) -> Result<Expr, ParseError> {
        if let TokenKind::Ident(sym) = self.peek().kind {
            let literal = match self.interner.resolve(sym) {
                "verum" => Some(Literal::Bool(true)),
                "falsum" => Some(Literal::Bool(false)),
                "nihil" => Some(Literal::Nil),
                _ => None,
            };

            if let Some(literal) = literal {
                let span = self.peek().span;
                self.advance();
                let id = self.next_id();
                return Ok(Expr { id, kind: ExprKind::Literal(literal), span });
            }
        }

        self.parse_expression()
    }

    fn is_annotation_bivalens_type(&self, ty: &TypeExpr) -> bool {
        matches!(
            &ty.kind,
            TypeExprKind::Named(name, params)
                if params.is_empty()
                    && !ty.nullable
                    && ty.mode.is_none()
                    && self.interner.resolve(name.name) == "bivalens"
        )
    }

    fn is_annotation_arg(&self) -> bool {
        if self.check(&TokenKind::At) {
            return false;
        }

        matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::Underscore(_)
                | TokenKind::String(_)
                | TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::Comma
                | TokenKind::Ceteri
                | TokenKind::De
                | TokenKind::Ex
                | TokenKind::Falsum
                | TokenKind::In
                | TokenKind::LParen
                | TokenKind::RParen
                | TokenKind::Arrow
                | TokenKind::ExitArrow
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Colon
                | TokenKind::Dot
                | TokenKind::Eq
                | TokenKind::Assign
                | TokenKind::Nihil
                | TokenKind::Si
                | TokenKind::Typus
                | TokenKind::Vel
                | TokenKind::Verum
        )
    }

    #[allow(dead_code)]
    fn parse_target_mappings(&mut self) -> Result<Vec<TargetMapping>, ParseError> {
        let mut mappings = Vec::new();

        loop {
            let start = self.current_span();
            let target = self.parse_ident()?;
            let value = self.parse_string()?;
            let span = start.merge(self.previous_span());

            mappings.push(TargetMapping { target, value, span });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(mappings)
    }

    // =============================================================================
    // BLOCK AND ANNOTATION PARSING
    // =============================================================================

    /// Parse a statement block and recover within its brace boundary.
    ///
    /// GRAMMAR:
    ///   block := '{' statement* '}'
    ///
    /// Block recovery passes `stop_at_rbrace = true` so a bad inner statement can
    /// be skipped without consuming the `}` that returns control to the caller.
    pub(super) fn parse_block(&mut self) -> Result<BlockStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize(true);
                }
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let span = start.merge(self.previous_span());
        Ok(BlockStmt { stmts, span })
    }

    /// Try to parse generic declaration parameters: `<T, U>`.
    fn try_parse_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if !self.eat(&TokenKind::Lt) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            let name = self.parse_ident()?;
            params.push(TypeParam { span: name.span, name });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::Gt, "expected '>'")?;
        Ok(params)
    }
}
