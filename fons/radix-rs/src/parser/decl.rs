//! Declaration parsing

use super::{Parser, ParseError, ParseErrorKind};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Parse a top-level or nested statement
    pub(super) fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        let id = self.next_id();

        let kind = match self.peek().kind {
            // Declarations
            TokenKind::Fixum | TokenKind::Varia | TokenKind::Figendum | TokenKind::Variandum => {
                self.parse_var_decl()?
            }
            TokenKind::Functio => self.parse_func_decl()?,
            TokenKind::Genus => self.parse_class_decl()?,
            TokenKind::Pactum => self.parse_interface_decl()?,
            TokenKind::Typus => self.parse_type_alias_decl()?,
            TokenKind::Ordo => self.parse_enum_decl()?,
            TokenKind::Discretio => self.parse_union_decl()?,
            TokenKind::Importa => self.parse_import_decl()?,
            TokenKind::Section => self.parse_directive_decl()?,
            TokenKind::Probandum => self.parse_test_decl()?,
            TokenKind::Abstractus => self.parse_abstract_class_decl()?,

            // Control flow
            TokenKind::Si => self.parse_if_stmt()?,
            TokenKind::Dum => self.parse_while_stmt()?,
            TokenKind::Itera => self.parse_iter_stmt()?,
            TokenKind::Elige => self.parse_switch_stmt()?,
            TokenKind::Discerne => self.parse_match_stmt()?,
            TokenKind::Custodi => self.parse_guard_stmt()?,
            TokenKind::Fac => self.parse_fac_stmt()?,

            // Transfer
            TokenKind::Redde => self.parse_return_stmt()?,
            TokenKind::Rumpe => self.parse_break_stmt()?,
            TokenKind::Perge => self.parse_continue_stmt()?,
            TokenKind::Iace => self.parse_throw_stmt()?,
            TokenKind::Mori => self.parse_panic_stmt()?,

            // Error handling
            TokenKind::Tempta => self.parse_try_stmt()?,
            TokenKind::Adfirma => self.parse_assert_stmt()?,

            // Output
            TokenKind::Scribe | TokenKind::Vide | TokenKind::Mone => self.parse_output_stmt()?,

            // Entry points
            TokenKind::Incipit | TokenKind::Incipiet => self.parse_entry_stmt()?,

            // Resource management
            TokenKind::Cura => self.parse_resource_stmt()?,

            // Endpoint
            TokenKind::Ad => self.parse_endpoint_stmt()?,

            // Block
            TokenKind::LBrace => StmtKind::Block(self.parse_block()?),

            // Annotations (for following declaration)
            TokenKind::At => {
                let _annotations = self.parse_annotations()?;
                // For now, just parse next statement
                // TODO: attach annotations
                return self.parse_statement();
            }

            // Expression statement
            _ => self.parse_expr_stmt()?,
        };

        let span = start.merge(self.previous_span());
        Ok(Stmt { id, kind, span })
    }

    /// Parse variable declaration
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
            TokenKind::Figendum => {
                self.advance();
                (Mutability::Immutable, true)
            }
            TokenKind::Variandum => {
                self.advance();
                (Mutability::Mutable, true)
            }
            _ => unreachable!(),
        };

        // Check: is this "name =" (no type) or "type... name =" (with type)?
        let ty = if self.is_simple_var_decl() {
            // Pattern: fixum name = ...
            None
        } else {
            // Pattern: fixum [type-annotation] name = ...
            Some(self.parse_type()?)
        };

        // Name
        let name = self.parse_ident()?;

        // Optional initializer
        let init = if self.eat(&TokenKind::Eq) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Var(VarDecl {
            mutability,
            is_await,
            ty,
            name,
            init,
        }))
    }

    /// Parse function declaration
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
            body,
            annotations: Vec::new(),
        }))
    }

    /// Parse parameter list
    fn parse_param_list(&mut self) -> Result<(Vec<TypeParam>, Vec<Param>), ParseError> {
        let mut type_params = Vec::new();
        let mut params = Vec::new();

        // Type parameters first (prae typus T)
        while self.check_keyword(TokenKind::Prae) {
            self.advance();
            self.expect_keyword(TokenKind::Typus, "expected 'typus' after 'prae'")?;
            let name = self.parse_ident()?;
            type_params.push(TypeParam {
                span: name.span,
                name,
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        // Regular parameters
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let start = self.current_span();

            // Optional: si
            let optional = self.eat_keyword(TokenKind::Si);

            // Mode: de/in/ex
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
            params.push(Param {
                optional,
                mode,
                rest,
                ty,
                name,
                alias,
                default,
                span,
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok((type_params, params))
    }

    /// Parse function modifiers
    fn parse_func_modifiers(&mut self) -> Result<Vec<FuncModifier>, ParseError> {
        let mut modifiers = Vec::new();

        loop {
            if self.eat_keyword(TokenKind::Curata) {
                let name = self.parse_ident()?;
                modifiers.push(FuncModifier::Curata(name));
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

    /// Parse class declaration
    fn parse_class_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.parse_class_decl_inner(false)
    }

    /// Parse abstract class declaration
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

    /// Parse class member
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

            let init = if self.eat(&TokenKind::Colon) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            ClassMemberKind::Field(FieldDecl {
                is_static,
                is_bound,
                ty,
                name,
                init,
            })
        };

        let span = start.merge(self.previous_span());
        Ok(ClassMember {
            annotations,
            kind,
            span,
        })
    }

    /// Parse interface declaration
    fn parse_interface_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Pactum, "expected 'pactum'")?;

        let name = self.parse_ident()?;
        let type_params = self.try_parse_type_params()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let start = self.current_span();

            self.expect_keyword(TokenKind::Functio, "expected 'functio'")?;
            let method_name = self.parse_ident()?;

            self.expect(&TokenKind::LParen, "expected '('")?;
            let (_, params) = self.parse_param_list()?;
            self.expect(&TokenKind::RParen, "expected ')'")?;

            let modifiers = self.parse_func_modifiers()?;

            let ret = if self.eat(&TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let span = start.merge(self.previous_span());
            methods.push(InterfaceMethod {
                name: method_name,
                params,
                modifiers,
                ret,
                span,
            });
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Interface(InterfaceDecl {
            name,
            type_params,
            methods,
        }))
    }

    /// Parse type alias
    fn parse_type_alias_decl(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Typus, "expected 'typus'")?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::Eq, "expected '='")?;
        let ty = self.parse_type()?;

        Ok(StmtKind::TypeAlias(TypeAliasDecl { name, ty }))
    }

    /// Parse enum declaration
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
            members.push(EnumMember {
                name: member_name,
                value,
                span,
            });

            // Optional comma
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Enum(EnumDecl { name, members }))
    }

    /// Parse tagged union declaration
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
                    fields.push(VariantField {
                        ty,
                        name: field_name,
                        span: field_span,
                    });
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBrace, "expected '}'")?;
                fields
            } else {
                Vec::new()
            };

            let span = start.merge(self.previous_span());
            variants.push(Variant {
                name: variant_name,
                fields,
                span,
            });

            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Union(UnionDecl {
            name,
            type_params,
            variants,
        }))
    }

    /// Parse import declaration
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
        Ok(StmtKind::Import(ImportDecl {
            path,
            visibility,
            kind,
            span,
        }))
    }

    /// Parse directive
    fn parse_directive_decl(&mut self) -> Result<StmtKind, ParseError> {
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
        Ok(StmtKind::Directive(DirectiveDecl { name, args, span }))
    }

    /// Parse test declaration
    fn parse_test_decl(&mut self) -> Result<StmtKind, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Probandum, "expected 'probandum'")?;

        let name = self.parse_string()?;

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let body = self.parse_test_body()?;

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let span = start.merge(self.previous_span());
        Ok(StmtKind::Test(TestDecl { name, body, span }))
    }

    fn parse_test_body(&mut self) -> Result<TestBody, ParseError> {
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
                        SetupKind::Before
                    }
                    TokenKind::Praeparabit => {
                        self.advance();
                        SetupKind::BeforeAll
                    }
                    TokenKind::Postpara => {
                        self.advance();
                        SetupKind::After
                    }
                    TokenKind::Postparabit => {
                        self.advance();
                        SetupKind::AfterAll
                    }
                    _ => unreachable!(),
                };
                let all = self.eat_keyword(TokenKind::Omnia);
                let body = self.parse_block()?;
                let span = start.merge(self.previous_span());
                setup.push(SetupBlock {
                    kind,
                    all,
                    body,
                    span,
                });
            } else if self.check_keyword(TokenKind::Probandum) {
                if let StmtKind::Test(test) = self.parse_test_decl()? {
                    nested.push(test);
                }
            } else if self.check_keyword(TokenKind::Proba) {
                let case = self.parse_test_case()?;
                tests.push(case);
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected test case or setup"));
            }
        }

        Ok(TestBody {
            setup,
            tests,
            nested,
        })
    }

    fn parse_test_case(&mut self) -> Result<TestCase, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Proba, "expected 'proba'")?;

        let mut modifiers = Vec::new();

        // Parse modifiers before string
        loop {
            if self.eat_keyword(TokenKind::Omitte) {
                let reason = self.parse_string()?;
                modifiers.push(TestModifier::Skip(reason));
            } else if self.eat_keyword(TokenKind::Futurum) {
                let reason = self.parse_string()?;
                modifiers.push(TestModifier::Future(reason));
            } else if self.eat_keyword(TokenKind::Solum) {
                modifiers.push(TestModifier::Only);
            } else if self.eat_keyword(TokenKind::Tag) {
                let tag = self.parse_string()?;
                modifiers.push(TestModifier::Tag(tag));
            } else if self.eat_keyword(TokenKind::Temporis) {
                if let TokenKind::Integer(n) = self.peek().kind {
                    self.advance();
                    modifiers.push(TestModifier::Timeout(n));
                }
            } else if self.eat_keyword(TokenKind::Metior) {
                modifiers.push(TestModifier::Bench);
            } else if self.eat_keyword(TokenKind::Repete) {
                if let TokenKind::Integer(n) = self.peek().kind {
                    self.advance();
                    modifiers.push(TestModifier::Repeat(n));
                }
            } else if self.eat_keyword(TokenKind::Fragilis) {
                if let TokenKind::Integer(n) = self.peek().kind {
                    self.advance();
                    modifiers.push(TestModifier::Flaky(n));
                }
            } else if self.eat_keyword(TokenKind::Requirit) {
                let req = self.parse_string()?;
                modifiers.push(TestModifier::Requires(req));
            } else if self.eat_keyword(TokenKind::SolumIn) {
                let env = self.parse_string()?;
                modifiers.push(TestModifier::OnlyIn(env));
            } else {
                break;
            }
        }

        let name = self.parse_string()?;
        let body = self.parse_block()?;

        let span = start.merge(self.previous_span());
        Ok(TestCase {
            modifiers,
            name,
            body,
            span,
        })
    }

    /// Parse annotations
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
        // Check for known annotation keywords
        if self.eat_keyword(TokenKind::Innatum) {
            let mappings = self.parse_target_mappings()?;
            return Ok(AnnotationKind::Innatum(mappings));
        }

        // For now, just parse as simple identifier list
        let mut idents = Vec::new();
        while let Some(ident) = self.try_parse_ident() {
            idents.push(ident);
        }

        if idents.is_empty() {
            return Err(self.error(ParseErrorKind::InvalidAnnotation, "expected annotation name"));
        }

        Ok(AnnotationKind::Simple(idents))
    }

    fn parse_target_mappings(&mut self) -> Result<Vec<TargetMapping>, ParseError> {
        let mut mappings = Vec::new();

        loop {
            let start = self.current_span();
            let target = self.parse_ident()?;
            let value = self.parse_string()?;
            let span = start.merge(self.previous_span());

            mappings.push(TargetMapping {
                target,
                value,
                span,
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(mappings)
    }

    /// Parse block statement
    pub(super) fn parse_block(&mut self) -> Result<BlockStmt, ParseError> {
        let start = self.current_span();
        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let span = start.merge(self.previous_span());
        Ok(BlockStmt { stmts, span })
    }

    /// Try to parse optional type parameters: <T, U>
    fn try_parse_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if !self.eat(&TokenKind::Lt) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            let name = self.parse_ident()?;
            params.push(TypeParam {
                span: name.span,
                name,
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::Gt, "expected '>'")?;
        Ok(params)
    }
}
