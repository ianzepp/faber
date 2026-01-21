use crate::{
    CompileError, Expr, Locus, Modulus, ObiectumProp, Param, Stmt, Token, Typus,
    TOKEN_EOF, TOKEN_IDENTIFIER, TOKEN_KEYWORD, TOKEN_NUMERUS, TOKEN_OPERATOR,
    TOKEN_PUNCTUATOR, TOKEN_TEXTUS,
};
use std::collections::HashMap;

/// Operator precedence for Pratt parser.
fn precedence() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert("=", 1);
    m.insert("+=", 1);
    m.insert("-=", 1);
    m.insert("*=", 1);
    m.insert("/=", 1);
    m.insert("vel", 2);
    m.insert("??", 2);
    m.insert("aut", 3);
    m.insert("||", 3);
    m.insert("et", 4);
    m.insert("&&", 4);
    m.insert("==", 5);
    m.insert("!=", 5);
    m.insert("===", 5);
    m.insert("!==", 5);
    m.insert("<", 6);
    m.insert(">", 6);
    m.insert("<=", 6);
    m.insert(">=", 6);
    m.insert("inter", 6);
    m.insert("intra", 6);
    m.insert("+", 7);
    m.insert("-", 7);
    m.insert("*", 8);
    m.insert("/", 8);
    m.insert("%", 8);
    m.insert("qua", 9);
    m.insert("innatum", 9);
    m.insert("novum", 9);
    m
}

fn unary_ops() -> HashMap<&'static str, ()> {
    let mut m = HashMap::new();
    m.insert("-", ());
    m.insert("!", ());
    m.insert("non", ());
    m.insert("nihil", ());
    m.insert("nonnihil", ());
    m.insert("positivum", ());
    m
}

fn assign_ops() -> HashMap<&'static str, ()> {
    let mut m = HashMap::new();
    m.insert("=", ());
    m.insert("+=", ());
    m.insert("-=", ());
    m.insert("*=", ());
    m.insert("/=", ());
    m
}

/// Parser for Faber source.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    filename: String,
    precedence: HashMap<&'static str, i32>,
    unary_ops: HashMap<&'static str, ()>,
    assign_ops: HashMap<&'static str, ()>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filename: impl Into<String>) -> Self {
        Self {
            tokens,
            pos: 0,
            filename: filename.into(),
            precedence: precedence(),
            unary_ops: unary_ops(),
            assign_ops: assign_ops(),
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let idx = self.pos + offset;
        if idx >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[idx]
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn check(&self, tag: &str, valor: Option<&str>) -> bool {
        let tok = self.peek(0);
        if tok.tag != tag {
            return false;
        }
        if let Some(v) = valor {
            if tok.valor != v {
                return false;
            }
        }
        true
    }

    fn match_token(&mut self, tag: &str, valor: Option<&str>) -> Option<Token> {
        if self.check(tag, valor) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn expect(&mut self, tag: &str, valor: Option<&str>) -> Result<Token, CompileError> {
        if let Some(tok) = self.match_token(tag, valor) {
            Ok(tok)
        } else {
            let got = self.peek(0);
            let msg = valor.unwrap_or(tag);
            Err(self.error(&format!("expected {}, got '{}'", msg, got.valor)))
        }
    }

    fn error(&self, msg: &str) -> CompileError {
        CompileError::new(msg, self.peek(0).locus, &self.filename)
    }

    fn expect_name(&mut self) -> Result<Token, CompileError> {
        let tok = self.peek(0);
        if tok.tag == TOKEN_IDENTIFIER || tok.tag == TOKEN_KEYWORD {
            Ok(self.advance())
        } else {
            Err(self.error(&format!("expected identifier, got '{}'", tok.valor)))
        }
    }

    fn check_name(&self) -> bool {
        let tok = self.peek(0);
        tok.tag == TOKEN_IDENTIFIER || tok.tag == TOKEN_KEYWORD
    }

    /// Main parse entry point.
    pub fn parse(&mut self) -> Result<Modulus, CompileError> {
        let mut corpus = Vec::new();
        while !self.check(TOKEN_EOF, None) {
            corpus.push(self.parse_stmt()?);
        }
        Ok(Modulus {
            locus: Locus {
                linea: 1,
                columna: 1,
                index: 0,
            },
            corpus,
        })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, CompileError> {
        let mut publica = false;
        let mut futura = false;
        let mut externa = false;

        while self.match_token(TOKEN_PUNCTUATOR, Some("@")).is_some() {
            let tok = self.peek(0);
            if tok.tag != TOKEN_IDENTIFIER && tok.tag != TOKEN_KEYWORD {
                return Err(self.error("expected annotation name"));
            }
            let anno = self.advance().valor;
            match anno.as_str() {
                "publicum" | "publica" => publica = true,
                "futura" => futura = true,
                "externa" => externa = true,
                _ => {
                    while !self.check(TOKEN_EOF, None)
                        && !self.check(TOKEN_PUNCTUATOR, Some("@"))
                        && !self.check(TOKEN_PUNCTUATOR, Some("§"))
                        && !self.is_declaration_keyword()
                    {
                        self.advance();
                    }
                }
            }
        }

        if self.match_token(TOKEN_PUNCTUATOR, Some("§")).is_some() {
            return self.parse_import();
        }

        let tok = self.peek(0);
        if tok.tag == TOKEN_KEYWORD {
            match tok.valor.as_str() {
                "varia" | "fixum" | "figendum" => return self.parse_varia(publica, externa),
                "ex" => return self.parse_ex_stmt(publica),
                "functio" => return self.parse_functio(publica, futura, externa),
                "genus" => return self.parse_genus(publica),
                "pactum" => return self.parse_pactum(publica),
                "ordo" => return self.parse_ordo(publica),
                "discretio" => return self.parse_discretio(publica),
                "si" => return self.parse_si(),
                "dum" => return self.parse_dum(),
                "fac" => return self.parse_fac(),
                "elige" => return self.parse_elige(),
                "discerne" => return self.parse_discerne(),
                "custodi" => return self.parse_custodi(),
                "tempta" => return self.parse_tempta(),
                "redde" => return self.parse_redde(),
                "iace" | "mori" => return self.parse_iace(),
                "scribe" | "vide" | "mone" => return self.parse_scribe(),
                "adfirma" => return self.parse_adfirma(),
                "rumpe" => return self.parse_rumpe(),
                "perge" => return self.parse_perge(),
                "incipit" | "incipiet" => return self.parse_incipit(),
                "probandum" => return self.parse_probandum(),
                "proba" => return self.parse_proba(),
                _ => {}
            }
        }

        if self.check(TOKEN_PUNCTUATOR, Some("{")) {
            return self.parse_massa();
        }

        self.parse_expressia_stmt()
    }

    fn parse_import(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("ex"))?;
        let fons = self.expect(TOKEN_TEXTUS, None)?.valor;
        self.expect(TOKEN_KEYWORD, Some("importa"))?;

        let mut specs = Vec::new();
        loop {
            let loc = self.peek(0).locus;
            let imported = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            let mut local = imported.clone();
            if self.match_token(TOKEN_KEYWORD, Some("ut")).is_some() {
                local = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            }
            specs.push(crate::ImportSpec {
                locus: loc,
                imported,
                local,
            });
            if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                break;
            }
        }

        Ok(Stmt::Importa {
            locus,
            fons,
            specs,
            totum: false,
            alias: None,
        })
    }

    fn parse_varia(&mut self, publica: bool, externa: bool) -> Result<Stmt, CompileError> {
        use crate::VariaSpecies;

        let locus = self.peek(0).locus;
        let kw = self.advance().valor;
        let species = match kw.as_str() {
            "figendum" => VariaSpecies::Figendum,
            "fixum" => VariaSpecies::Fixum,
            _ => VariaSpecies::Varia,
        };

        let first = self.expect_name()?.valor;

        let (typus, nomen) = if self.check(TOKEN_OPERATOR, Some("<")) {
            self.advance();
            let mut args = Vec::new();
            loop {
                args.push(self.parse_typus()?);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
            let mut t = Typus::Genericus { nomen: first, args };
            if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                t = Typus::Nullabilis { inner: Box::new(t) };
            }
            let n = self.expect_name()?.valor;
            (Some(t), n)
        } else if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
            let t = Typus::Nullabilis {
                inner: Box::new(Typus::Nomen { nomen: first }),
            };
            let n = self.expect_name()?.valor;
            (Some(t), n)
        } else if self.check_name() {
            let t = Typus::Nomen { nomen: first };
            let n = self.expect_name()?.valor;
            (Some(t), n)
        } else if self.match_token(TOKEN_PUNCTUATOR, Some(":")).is_some() {
            let t = self.parse_typus()?;
            (Some(t), first)
        } else {
            (None, first)
        };

        let valor = if self.match_token(TOKEN_OPERATOR, Some("=")).is_some() {
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        Ok(Stmt::Varia {
            locus,
            species,
            nomen,
            typus,
            valor,
            publica,
            externa,
        })
    }

    fn parse_ex_stmt(&mut self, _publica: bool) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("ex"))?;
        let expr = self.parse_expr(0)?;

        if self.check(TOKEN_KEYWORD, Some("fixum")) || self.check(TOKEN_KEYWORD, Some("varia")) {
            self.advance();
            let binding = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            let corpus = Box::new(self.parse_massa()?);
            return Ok(Stmt::Iteratio {
                locus,
                species: "Ex".to_string(),
                binding,
                iter: expr,
                corpus,
                asynca: false,
            });
        }

        Err(self.error("destructuring not supported in nanus"))
    }

    fn parse_functio(
        &mut self,
        publica: bool,
        futura: bool,
        externa: bool,
    ) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("functio"))?;
        let asynca = futura;

        let nomen = self.expect_name()?.valor;

        let mut generics = Vec::new();
        if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
            loop {
                generics.push(self.expect(TOKEN_IDENTIFIER, None)?.valor);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
        }

        self.expect(TOKEN_PUNCTUATOR, Some("("))?;
        let params = self.parse_params()?;
        self.expect(TOKEN_PUNCTUATOR, Some(")"))?;

        let typus_reditus = if self.match_token(TOKEN_OPERATOR, Some("->")).is_some() {
            Some(self.parse_typus()?)
        } else {
            None
        };

        let corpus = if self.check(TOKEN_PUNCTUATOR, Some("{")) {
            Some(Box::new(self.parse_massa()?))
        } else {
            None
        };

        Ok(Stmt::Functio {
            locus,
            nomen,
            params,
            typus_reditus,
            corpus,
            asynca,
            publica,
            generics,
            externa,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, CompileError> {
        let mut params = Vec::new();
        if self.check(TOKEN_PUNCTUATOR, Some(")")) {
            return Ok(params);
        }

        loop {
            let locus = self.peek(0).locus;
            let rest = self.match_token(TOKEN_KEYWORD, Some("ceteri")).is_some();
            let optional = self.match_token(TOKEN_KEYWORD, Some("si")).is_some();
            // Skip ownership annotations (recognized but not enforced yet)
            self.match_token(TOKEN_KEYWORD, Some("ex"));
            self.match_token(TOKEN_KEYWORD, Some("de"));

            if !self.check_name() {
                return Err(self.error("expected parameter name"));
            }

            let first = self.expect_name()?.valor;

            let (mut typus, nomen) = if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
                let mut args = Vec::new();
                loop {
                    args.push(self.parse_typus()?);
                    if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                        break;
                    }
                }
                self.expect(TOKEN_OPERATOR, Some(">"))?;
                let mut t = Typus::Genericus { nomen: first, args };
                if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                    t = Typus::Nullabilis { inner: Box::new(t) };
                }
                let n = self.expect_name()?.valor;
                (Some(t), n)
            } else if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                let t = Typus::Nullabilis {
                    inner: Box::new(Typus::Nomen { nomen: first }),
                };
                let n = self.expect_name()?.valor;
                (Some(t), n)
            } else if self.check_name() {
                let t = Typus::Nomen { nomen: first };
                let n = self.expect_name()?.valor;
                (Some(t), n)
            } else if self.match_token(TOKEN_PUNCTUATOR, Some(":")).is_some() {
                let t = self.parse_typus()?;
                (Some(t), first)
            } else {
                (None, first)
            };

            if optional {
                if let Some(t) = typus {
                    if !matches!(t, Typus::Nullabilis { .. }) {
                        typus = Some(Typus::Nullabilis { inner: Box::new(t) });
                    } else {
                        typus = Some(t);
                    }
                }
            }

            let default = if self.match_token(TOKEN_OPERATOR, Some("=")).is_some() {
                Some(self.parse_expr(0)?)
            } else {
                None
            };

            params.push(Param {
                locus,
                nomen,
                typus,
                default,
                rest,
            });

            if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                break;
            }
        }

        Ok(params)
    }

    fn parse_genus(&mut self, publica: bool) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("genus"))?;
        let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;

        let mut generics = Vec::new();
        if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
            loop {
                generics.push(self.expect(TOKEN_IDENTIFIER, None)?.valor);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
        }

        let mut implet = Vec::new();
        if self.match_token(TOKEN_KEYWORD, Some("implet")).is_some() {
            loop {
                implet.push(self.expect(TOKEN_IDENTIFIER, None)?.valor);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
        }

        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut campi = Vec::new();
        let mut methodi = Vec::new();

        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            while self.match_token(TOKEN_PUNCTUATOR, Some("@")).is_some() {
                let tok = self.peek(0);
                if tok.tag != TOKEN_IDENTIFIER && tok.tag != TOKEN_KEYWORD {
                    return Err(self.error("expected annotation name"));
                }
                self.advance();
            }

            let mut visibilitas = "Publica".to_string();
            if self.match_token(TOKEN_KEYWORD, Some("privata")).is_some()
                || self.match_token(TOKEN_KEYWORD, Some("privatus")).is_some()
            {
                visibilitas = "Privata".to_string();
            } else if self.match_token(TOKEN_KEYWORD, Some("protecta")).is_some()
                || self.match_token(TOKEN_KEYWORD, Some("protectus")).is_some()
            {
                visibilitas = "Protecta".to_string();
            }

            if self.check(TOKEN_KEYWORD, Some("functio")) {
                methodi.push(self.parse_functio(false, false, false)?);
            } else {
                let loc = self.peek(0).locus;
                let first = self.expect_name()?.valor;

                let (field_typus, field_nomen) = if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_typus()?);
                        if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                            break;
                        }
                    }
                    self.expect(TOKEN_OPERATOR, Some(">"))?;
                    let mut t = Typus::Genericus { nomen: first, args };
                    if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                        t = Typus::Nullabilis { inner: Box::new(t) };
                    }
                    let n = self.expect_name()?.valor;
                    (Some(t), n)
                } else {
                    let nullable = self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some();
                    if self.check_name() {
                        let mut t = Typus::Nomen { nomen: first };
                        if nullable {
                            t = Typus::Nullabilis { inner: Box::new(t) };
                        }
                        let n = self.expect_name()?.valor;
                        (Some(t), n)
                    } else if self.match_token(TOKEN_PUNCTUATOR, Some(":")).is_some() {
                        let t = self.parse_typus()?;
                        (Some(t), first)
                    } else {
                        return Err(self.error("expected field type or name"));
                    }
                };

                let valor = if self.match_token(TOKEN_OPERATOR, Some("=")).is_some() {
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };

                campi.push(crate::CampusDecl {
                    locus: loc,
                    nomen: field_nomen,
                    typus: field_typus,
                    valor,
                    visibilitas,
                });
            }
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Genus {
            locus,
            nomen,
            campi,
            methodi,
            implet,
            generics,
            publica,
        })
    }

    fn parse_pactum(&mut self, publica: bool) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("pactum"))?;
        let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;

        let mut generics = Vec::new();
        if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
            loop {
                generics.push(self.expect(TOKEN_IDENTIFIER, None)?.valor);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
        }

        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut methodi = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            let loc = self.peek(0).locus;
            self.expect(TOKEN_KEYWORD, Some("functio"))?;
            let asynca = self.match_token(TOKEN_KEYWORD, Some("asynca")).is_some();
            let name = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            self.expect(TOKEN_PUNCTUATOR, Some("("))?;
            let params = self.parse_params()?;
            self.expect(TOKEN_PUNCTUATOR, Some(")"))?;
            let typus_reditus = if self.match_token(TOKEN_OPERATOR, Some("->")).is_some() {
                Some(self.parse_typus()?)
            } else {
                None
            };
            methodi.push(crate::PactumMethodus {
                locus: loc,
                nomen: name,
                params,
                typus_reditus,
                asynca,
            });
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Pactum {
            locus,
            nomen,
            methodi,
            generics,
            publica,
        })
    }

    fn parse_ordo(&mut self, publica: bool) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("ordo"))?;
        let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut membra = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            let loc = self.peek(0).locus;
            let name = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            let valor = if self.match_token(TOKEN_OPERATOR, Some("=")).is_some() {
                let tok = self.peek(0);
                let v = if tok.tag == TOKEN_TEXTUS {
                    format!("\"{}\"", tok.valor)
                } else {
                    tok.valor.clone()
                };
                self.advance();
                Some(v)
            } else {
                None
            };
            membra.push(crate::OrdoMembrum {
                locus: loc,
                nomen: name,
                valor,
            });
            self.match_token(TOKEN_PUNCTUATOR, Some(","));
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Ordo {
            locus,
            nomen,
            membra,
            publica,
        })
    }

    fn parse_discretio(&mut self, publica: bool) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("discretio"))?;
        let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;

        let mut generics = Vec::new();
        if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
            loop {
                generics.push(self.expect(TOKEN_IDENTIFIER, None)?.valor);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
        }

        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut variantes = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            let loc = self.peek(0).locus;
            let name = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            let mut campi = Vec::new();

            if self.match_token(TOKEN_PUNCTUATOR, Some("{")).is_some() {
                while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
                    let typ_nomen = self.expect_name()?.valor;
                    let field_typus = if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
                        let mut args = Vec::new();
                        loop {
                            args.push(self.parse_typus()?);
                            if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                                break;
                            }
                        }
                        self.expect(TOKEN_OPERATOR, Some(">"))?;
                        let mut t = Typus::Genericus {
                            nomen: typ_nomen,
                            args,
                        };
                        if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                            t = Typus::Nullabilis { inner: Box::new(t) };
                        }
                        t
                    } else {
                        let mut t = Typus::Nomen { nomen: typ_nomen };
                        if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
                            t = Typus::Nullabilis { inner: Box::new(t) };
                        }
                        t
                    };
                    let field_nomen = self.expect_name()?.valor;
                    campi.push(crate::VariansCampus {
                        nomen: field_nomen,
                        typus: field_typus,
                    });
                }
                self.expect(TOKEN_PUNCTUATOR, Some("}"))?;
            }

            variantes.push(crate::VariansDecl {
                locus: loc,
                nomen: name,
                campi,
            });
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Discretio {
            locus,
            nomen,
            variantes,
            generics,
            publica,
        })
    }

    fn parse_massa(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;
        let mut corpus = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            corpus.push(self.parse_stmt()?);
        }
        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;
        Ok(Stmt::Massa { locus, corpus })
    }

    fn parse_body(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;

        if self.check(TOKEN_PUNCTUATOR, Some("{")) {
            return self.parse_massa();
        }

        if self.match_token(TOKEN_KEYWORD, Some("ergo")).is_some() {
            let stmt = self.parse_stmt()?;
            return Ok(Stmt::Massa {
                locus,
                corpus: vec![stmt],
            });
        }

        if self.match_token(TOKEN_KEYWORD, Some("reddit")).is_some() {
            let valor = self.parse_expr(0)?;
            return Ok(Stmt::Massa {
                locus,
                corpus: vec![Stmt::Redde {
                    locus,
                    valor: Some(valor),
                }],
            });
        }

        if self.match_token(TOKEN_KEYWORD, Some("iacit")).is_some() {
            let arg = self.parse_expr(0)?;
            return Ok(Stmt::Massa {
                locus,
                corpus: vec![Stmt::Iace {
                    locus,
                    arg,
                    fatale: false,
                }],
            });
        }

        if self.match_token(TOKEN_KEYWORD, Some("moritor")).is_some() {
            let arg = self.parse_expr(0)?;
            return Ok(Stmt::Massa {
                locus,
                corpus: vec![Stmt::Iace {
                    locus,
                    arg,
                    fatale: true,
                }],
            });
        }

        if self.match_token(TOKEN_KEYWORD, Some("tacet")).is_some() {
            return Ok(Stmt::Massa {
                locus,
                corpus: vec![],
            });
        }

        self.parse_massa()
    }

    fn parse_si(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("si"))?;
        self.parse_si_body(locus)
    }

    fn parse_si_body(&mut self, locus: Locus) -> Result<Stmt, CompileError> {
        let cond = self.parse_expr(0)?;
        let cons = Box::new(self.parse_body()?);

        let alt = if self.match_token(TOKEN_KEYWORD, Some("sin")).is_some() {
            let sin_locus = self.peek(0).locus;
            Some(Box::new(self.parse_si_body(sin_locus)?))
        } else if self.match_token(TOKEN_KEYWORD, Some("secus")).is_some() {
            if self.check(TOKEN_KEYWORD, Some("si")) {
                Some(Box::new(self.parse_si()?))
            } else {
                Some(Box::new(self.parse_body()?))
            }
        } else {
            None
        };

        Ok(Stmt::Si { locus, cond, cons, alt })
    }

    fn parse_dum(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("dum"))?;
        let cond = self.parse_expr(0)?;
        let corpus = Box::new(self.parse_body()?);
        Ok(Stmt::Dum { locus, cond, corpus })
    }

    fn parse_fac(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("fac"))?;
        let corpus = Box::new(self.parse_massa()?);
        self.expect(TOKEN_KEYWORD, Some("dum"))?;
        let cond = self.parse_expr(0)?;
        Ok(Stmt::FacDum { locus, corpus, cond })
    }

    fn parse_elige(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("elige"))?;
        let discrim = self.parse_expr(0)?;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut casus = Vec::new();
        let mut default = None;

        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            if self.match_token(TOKEN_KEYWORD, Some("ceterum")).is_some() {
                if self.check(TOKEN_PUNCTUATOR, Some("{")) {
                    default = Some(Box::new(self.parse_massa()?));
                } else if self.match_token(TOKEN_KEYWORD, Some("reddit")).is_some() {
                    let red_loc = self.peek(0).locus;
                    let valor = self.parse_expr(0)?;
                    default = Some(Box::new(Stmt::Massa {
                        locus: red_loc,
                        corpus: vec![Stmt::Redde {
                            locus: red_loc,
                            valor: Some(valor),
                        }],
                    }));
                } else {
                    return Err(self.error("expected { or reddit after ceterum"));
                }
            } else {
                self.expect(TOKEN_KEYWORD, Some("casu"))?;
                let loc = self.peek(0).locus;
                let cond = self.parse_expr(0)?;
                let corpus = if self.check(TOKEN_PUNCTUATOR, Some("{")) {
                    self.parse_massa()?
                } else if self.match_token(TOKEN_KEYWORD, Some("reddit")).is_some() {
                    let red_loc = self.peek(0).locus;
                    let valor = self.parse_expr(0)?;
                    Stmt::Massa {
                        locus: red_loc,
                        corpus: vec![Stmt::Redde {
                            locus: red_loc,
                            valor: Some(valor),
                        }],
                    }
                } else {
                    return Err(self.error("expected { or reddit after casu condition"));
                };
                casus.push(crate::EligeCasus { locus: loc, cond, corpus: Box::new(corpus) });
            }
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Elige {
            locus,
            discrim,
            casus,
            default,
        })
    }

    fn parse_discerne(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("discerne"))?;
        let mut discrim = vec![self.parse_expr(0)?];
        while self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_some() {
            discrim.push(self.parse_expr(0)?);
        }
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut casus = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            let loc = self.peek(0).locus;

            if self.match_token(TOKEN_KEYWORD, Some("ceterum")).is_some() {
                let patterns = vec![crate::VariansPattern {
                    locus: loc,
                    variant: "_".to_string(),
                    bindings: vec![],
                    alias: None,
                    wildcard: true,
                }];
                let corpus = self.parse_massa()?;
                casus.push(crate::DiscerneCasus { locus: loc, patterns, corpus: Box::new(corpus) });
                continue;
            }

            self.expect(TOKEN_KEYWORD, Some("casu"))?;
            let mut patterns = Vec::new();

            loop {
                let p_loc = self.peek(0).locus;
                let variant = self.expect(TOKEN_IDENTIFIER, None)?.valor;
                let wildcard = variant == "_";
                let mut alias = None;
                let mut bindings = Vec::new();

                if self.match_token(TOKEN_KEYWORD, Some("ut")).is_some() {
                    alias = Some(self.expect_name()?.valor);
                } else if self.match_token(TOKEN_KEYWORD, Some("pro")).is_some()
                    || self.match_token(TOKEN_KEYWORD, Some("fixum")).is_some()
                {
                    loop {
                        bindings.push(self.expect_name()?.valor);
                        if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                            break;
                        }
                    }
                }

                patterns.push(crate::VariansPattern {
                    locus: p_loc,
                    variant,
                    bindings,
                    alias,
                    wildcard,
                });

                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }

            let corpus = self.parse_massa()?;
            casus.push(crate::DiscerneCasus { locus: loc, patterns, corpus: Box::new(corpus) });
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Discerne { locus, discrim, casus })
    }

    fn parse_custodi(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("custodi"))?;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut clausulae = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            let loc = self.peek(0).locus;
            self.expect(TOKEN_KEYWORD, Some("si"))?;
            let cond = self.parse_expr(0)?;
            let corpus = self.parse_massa()?;
            clausulae.push(crate::CustodiClausula { locus: loc, cond, corpus: Box::new(corpus) });
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        Ok(Stmt::Custodi { locus, clausulae })
    }

    fn parse_tempta(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("tempta"))?;
        let corpus = Box::new(self.parse_massa()?);

        let cape = if self.match_token(TOKEN_KEYWORD, Some("cape")).is_some() {
            let loc = self.peek(0).locus;
            let param = self.expect(TOKEN_IDENTIFIER, None)?.valor;
            let body = self.parse_massa()?;
            Some(crate::CapeClausula {
                locus: loc,
                param,
                corpus: Box::new(body),
            })
        } else {
            None
        };

        let demum = if self.match_token(TOKEN_KEYWORD, Some("demum")).is_some() {
            Some(Box::new(self.parse_massa()?))
        } else {
            None
        };

        Ok(Stmt::Tempta { locus, corpus, cape, demum })
    }

    fn parse_redde(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("redde"))?;
        let valor =
            if !self.check(TOKEN_EOF, None) && !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.is_statement_keyword()
            {
                Some(self.parse_expr(0)?)
            } else {
                None
            };
        Ok(Stmt::Redde { locus, valor })
    }

    fn is_statement_keyword(&self) -> bool {
        if !self.check(TOKEN_KEYWORD, None) {
            return false;
        }
        let kw = &self.peek(0).valor;
        matches!(
            kw.as_str(),
            "si" | "sin"
                | "secus"
                | "dum"
                | "fac"
                | "ex"
                | "de"
                | "elige"
                | "discerne"
                | "custodi"
                | "tempta"
                | "cape"
                | "demum"
                | "redde"
                | "rumpe"
                | "perge"
                | "iace"
                | "mori"
                | "scribe"
                | "vide"
                | "mone"
                | "adfirma"
                | "functio"
                | "genus"
                | "pactum"
                | "ordo"
                | "discretio"
                | "varia"
                | "fixum"
                | "figendum"
                | "incipit"
                | "probandum"
                | "proba"
                | "casu"
                | "ceterum"
                | "reddit"
                | "ergo"
                | "tacet"
                | "iacit"
                | "moritor"
        )
    }

    fn is_declaration_keyword(&self) -> bool {
        if !self.check(TOKEN_KEYWORD, None) {
            return false;
        }
        let kw = &self.peek(0).valor;
        matches!(
            kw.as_str(),
            "functio" | "genus" | "pactum" | "ordo" | "discretio" | "varia" | "fixum" | "figendum" | "incipit" | "probandum"
        )
    }

    fn parse_iace(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        let fatale = self.advance().valor == "mori";
        let arg = self.parse_expr(0)?;
        Ok(Stmt::Iace { locus, arg, fatale })
    }

    fn parse_scribe(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        let kw = self.advance().valor;
        let gradus = match kw.as_str() {
            "vide" => "Vide",
            "mone" => "Mone",
            _ => "Scribe",
        }
        .to_string();

        let mut args = Vec::new();
        if !self.check(TOKEN_EOF, None) && !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.is_statement_keyword() {
            loop {
                args.push(self.parse_expr(0)?);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
        }
        Ok(Stmt::Scribe { locus, gradus, args })
    }

    fn parse_adfirma(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("adfirma"))?;
        let cond = self.parse_expr(0)?;
        let msg = if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_some() {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        Ok(Stmt::Adfirma { locus, cond, msg })
    }

    fn parse_rumpe(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("rumpe"))?;
        Ok(Stmt::Rumpe { locus })
    }

    fn parse_perge(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("perge"))?;
        Ok(Stmt::Perge { locus })
    }

    fn parse_incipit(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        let kw = self.advance().valor;
        let asynca = kw == "incipiet";
        let corpus = Box::new(self.parse_massa()?);
        Ok(Stmt::Incipit { locus, corpus, asynca })
    }

    fn parse_probandum(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("probandum"))?;
        let nomen = self.expect(TOKEN_TEXTUS, None)?.valor;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut corpus = Vec::new();
        while !self.check(TOKEN_PUNCTUATOR, Some("}")) && !self.check(TOKEN_EOF, None) {
            corpus.push(self.parse_stmt()?);
        }

        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;
        Ok(Stmt::Probandum { locus, nomen, corpus })
    }

    fn parse_proba(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("proba"))?;
        let nomen = self.expect(TOKEN_TEXTUS, None)?.valor;
        let corpus = Box::new(self.parse_massa()?);
        Ok(Stmt::Proba { locus, nomen, corpus })
    }

    fn parse_expressia_stmt(&mut self) -> Result<Stmt, CompileError> {
        let locus = self.peek(0).locus;
        let expr = self.parse_expr(0)?;
        Ok(Stmt::Expressia { locus, expr })
    }

    fn parse_typus(&mut self) -> Result<Typus, CompileError> {
        let mut typus = self.parse_typus_primary()?;

        if self.match_token(TOKEN_PUNCTUATOR, Some("?")).is_some() {
            typus = Typus::Nullabilis {
                inner: Box::new(typus),
            };
        }

        if self.match_token(TOKEN_OPERATOR, Some("|")).is_some() {
            let mut members = vec![typus];
            loop {
                members.push(self.parse_typus_primary()?);
                if self.match_token(TOKEN_OPERATOR, Some("|")).is_none() {
                    break;
                }
            }
            typus = Typus::Unio { members };
        }

        Ok(typus)
    }

    fn parse_typus_primary(&mut self) -> Result<Typus, CompileError> {
        let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;

        if self.match_token(TOKEN_OPERATOR, Some("<")).is_some() {
            let mut args = Vec::new();
            loop {
                args.push(self.parse_typus()?);
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
            self.expect(TOKEN_OPERATOR, Some(">"))?;
            return Ok(Typus::Genericus { nomen, args });
        }

        Ok(Typus::Nomen { nomen })
    }

    fn parse_expr(&mut self, min_prec: i32) -> Result<Expr, CompileError> {
        let mut left = self.parse_unary()?;

        loop {
            let tok = self.peek(0).clone();
            let op = &tok.valor;
            let prec = self.precedence.get(op.as_str()).copied().unwrap_or(-1);
            if prec < min_prec {
                break;
            }

            self.advance();

            if op == "qua" {
                let typus = self.parse_typus()?;
                left = Expr::Qua {
                    locus: tok.locus,
                    expr: Box::new(left),
                    typus,
                };
                continue;
            }
            if op == "innatum" {
                let typus = self.parse_typus()?;
                left = Expr::Innatum {
                    locus: tok.locus,
                    expr: Box::new(left),
                    typus,
                };
                continue;
            }
            if op == "novum" {
                let typus = self.parse_typus()?;
                left = Expr::PostfixNovum {
                    locus: tok.locus,
                    expr: Box::new(left),
                    typus,
                };
                continue;
            }

            let right = self.parse_expr(prec + 1)?;

            if self.assign_ops.contains_key(op.as_str()) {
                left = Expr::Assignatio {
                    locus: tok.locus,
                    signum: op.clone(),
                    sin: Box::new(left),
                    dex: Box::new(right),
                };
            } else {
                left = Expr::Binaria {
                    locus: tok.locus,
                    signum: op.clone(),
                    sin: Box::new(left),
                    dex: Box::new(right),
                };
            }
        }

        if self.match_token(TOKEN_KEYWORD, Some("sic")).is_some() {
            let cons = self.parse_expr(0)?;
            self.expect(TOKEN_KEYWORD, Some("secus"))?;
            let alt = self.parse_expr(0)?;
            left = Expr::Condicio {
                locus: left.locus(),
                cond: Box::new(left),
                cons: Box::new(cons),
                alt: Box::new(alt),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, CompileError> {
        let tok = self.peek(0).clone();

        if tok.tag == TOKEN_OPERATOR || tok.tag == TOKEN_KEYWORD {
            if self.unary_ops.contains_key(tok.valor.as_str()) {
                let non_expr: std::collections::HashSet<&str> = [
                    "qua", "innatum", "et", "aut", "vel", "sic", "secus", "inter", "intra", "perge",
                    "rumpe", "redde", "reddit", "iace", "mori", "si", "secussi", "dum", "ex", "de",
                    "elige", "discerne", "custodi", "tempta", "functio", "genus", "pactum", "ordo",
                    "discretio", "casu", "ceterum", "importa", "incipit", "incipiet", "probandum",
                    "proba",
                ]
                .into_iter()
                .collect();

                let next = self.peek(1);
                let can_be_unary = next.tag == TOKEN_IDENTIFIER
                    || (next.tag == TOKEN_KEYWORD && !non_expr.contains(next.valor.as_str()))
                    || next.tag == TOKEN_NUMERUS
                    || next.tag == TOKEN_TEXTUS
                    || next.valor == "("
                    || next.valor == "["
                    || next.valor == "{"
                    || self.unary_ops.contains_key(next.valor.as_str());

                if can_be_unary {
                    self.advance();
                    let arg = self.parse_unary()?;
                    return Ok(Expr::Unaria {
                        locus: tok.locus,
                        signum: tok.valor,
                        arg: Box::new(arg),
                    });
                }
            }
        }

        if self.match_token(TOKEN_KEYWORD, Some("cede")).is_some() {
            let arg = self.parse_unary()?;
            return Ok(Expr::Cede {
                locus: tok.locus,
                arg: Box::new(arg),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary()?;

        loop {
            let tok = self.peek(0).clone();

            if self.match_token(TOKEN_PUNCTUATOR, Some("(")).is_some() {
                let args = self.parse_args()?;
                self.expect(TOKEN_PUNCTUATOR, Some(")"))?;
                expr = Expr::Vocatio {
                    locus: tok.locus,
                    callee: Box::new(expr),
                    args,
                };
                continue;
            }

            if self.match_token(TOKEN_PUNCTUATOR, Some(".")).is_some() {
                let prop_name = self.expect_name()?.valor;
                let prop = Expr::Littera {
                    locus: self.peek(0).locus,
                    species: crate::LitteraSpecies::Textus,
                    valor: prop_name,
                };
                expr = Expr::Membrum {
                    locus: tok.locus,
                    obj: Box::new(expr),
                    prop: Box::new(prop),
                    computed: false,
                    non_null: false,
                };
                continue;
            }

            if tok.valor == "!" && self.peek(1).valor == "." {
                self.advance();
                self.advance();
                let prop_name = self.expect_name()?.valor;
                let prop = Expr::Littera {
                    locus: self.peek(0).locus,
                    species: crate::LitteraSpecies::Textus,
                    valor: prop_name,
                };
                expr = Expr::Membrum {
                    locus: tok.locus,
                    obj: Box::new(expr),
                    prop: Box::new(prop),
                    computed: false,
                    non_null: true,
                };
                continue;
            }

            if tok.valor == "!" && self.peek(1).valor == "[" {
                self.advance();
                self.advance();
                let prop = self.parse_expr(0)?;
                self.expect(TOKEN_PUNCTUATOR, Some("]"))?;
                expr = Expr::Membrum {
                    locus: tok.locus,
                    obj: Box::new(expr),
                    prop: Box::new(prop),
                    computed: true,
                    non_null: true,
                };
                continue;
            }

            if self.match_token(TOKEN_PUNCTUATOR, Some("[")).is_some() {
                let prop = self.parse_expr(0)?;
                self.expect(TOKEN_PUNCTUATOR, Some("]"))?;
                expr = Expr::Membrum {
                    locus: tok.locus,
                    obj: Box::new(expr),
                    prop: Box::new(prop),
                    computed: true,
                    non_null: false,
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CompileError> {
        use crate::LitteraSpecies;

        let tok = self.peek(0).clone();

        if self.match_token(TOKEN_PUNCTUATOR, Some("(")).is_some() {
            let expr = self.parse_expr(0)?;
            self.expect(TOKEN_PUNCTUATOR, Some(")"))?;
            return Ok(expr);
        }

        if self.match_token(TOKEN_PUNCTUATOR, Some("[")).is_some() {
            let mut elementa = Vec::new();
            if !self.check(TOKEN_PUNCTUATOR, Some("]")) {
                loop {
                    elementa.push(self.parse_expr(0)?);
                    if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                        break;
                    }
                }
            }
            self.expect(TOKEN_PUNCTUATOR, Some("]"))?;
            return Ok(Expr::Series {
                locus: tok.locus,
                elementa,
            });
        }

        if self.match_token(TOKEN_PUNCTUATOR, Some("{")).is_some() {
            let mut props = Vec::new();
            if !self.check(TOKEN_PUNCTUATOR, Some("}")) {
                loop {
                    let loc = self.peek(0).locus;
                    let (key, computed) = if self.match_token(TOKEN_PUNCTUATOR, Some("[")).is_some()
                    {
                        let k = self.parse_expr(0)?;
                        self.expect(TOKEN_PUNCTUATOR, Some("]"))?;
                        (k, true)
                    } else if self.check(TOKEN_TEXTUS, None) {
                        let str_key = self.advance().valor;
                        (
                            Expr::Littera {
                                locus: loc,
                                species: LitteraSpecies::Textus,
                                valor: str_key,
                            },
                            false,
                        )
                    } else {
                        let name = self.expect_name()?.valor;
                        (
                            Expr::Littera {
                                locus: loc,
                                species: LitteraSpecies::Textus,
                                valor: name,
                            },
                            false,
                        )
                    };

                    let (valor, shorthand) = if self.match_token(TOKEN_PUNCTUATOR, Some(":")).is_some() {
                        (self.parse_expr(0)?, false)
                    } else {
                        let key_name = match &key {
                            Expr::Littera { valor, .. } => valor.clone(),
                            _ => String::new(),
                        };
                        (
                            Expr::Nomen {
                                locus: loc,
                                valor: key_name,
                            },
                            true,
                        )
                    };

                    props.push(ObiectumProp {
                        locus: loc,
                        key,
                        valor,
                        shorthand,
                        computed,
                    });

                    if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                        break;
                    }
                }
            }
            self.expect(TOKEN_PUNCTUATOR, Some("}"))?;
            return Ok(Expr::Obiectum {
                locus: tok.locus,
                props,
            });
        }

        if tok.tag == TOKEN_KEYWORD {
            match tok.valor.as_str() {
                "verum" => {
                    self.advance();
                    return Ok(Expr::Littera {
                        locus: tok.locus,
                        species: LitteraSpecies::Verum,
                        valor: "true".to_string(),
                    });
                }
                "falsum" => {
                    self.advance();
                    return Ok(Expr::Littera {
                        locus: tok.locus,
                        species: LitteraSpecies::Falsum,
                        valor: "false".to_string(),
                    });
                }
                "nihil" => {
                    self.advance();
                    return Ok(Expr::Littera {
                        locus: tok.locus,
                        species: LitteraSpecies::Nihil,
                        valor: "null".to_string(),
                    });
                }
                "ego" => {
                    self.advance();
                    return Ok(Expr::Ego { locus: tok.locus });
                }
                "novum" => return self.parse_novum(),
                "finge" => return self.parse_finge(),
                "clausura" => return self.parse_clausura(),
                "scriptum" => return self.parse_scriptum(),
                _ => {
                    self.advance();
                    return Ok(Expr::Nomen {
                        locus: tok.locus,
                        valor: tok.valor,
                    });
                }
            }
        }

        if tok.tag == TOKEN_NUMERUS {
            self.advance();
            let species = if tok.valor.contains('.') {
                LitteraSpecies::Fractus
            } else {
                LitteraSpecies::Numerus
            };
            return Ok(Expr::Littera {
                locus: tok.locus,
                species,
                valor: tok.valor,
            });
        }

        if tok.tag == TOKEN_TEXTUS {
            self.advance();
            return Ok(Expr::Littera {
                locus: tok.locus,
                species: LitteraSpecies::Textus,
                valor: tok.valor,
            });
        }

        if tok.tag == TOKEN_IDENTIFIER {
            self.advance();
            return Ok(Expr::Nomen {
                locus: tok.locus,
                valor: tok.valor,
            });
        }

        Err(self.error(&format!("unexpected token '{}'", tok.valor)))
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, CompileError> {
        let mut args = Vec::new();
        if self.check(TOKEN_PUNCTUATOR, Some(")")) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr(0)?);
            if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                break;
            }
        }

        Ok(args)
    }

    fn parse_novum(&mut self) -> Result<Expr, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("novum"))?;
        let callee = self.parse_primary()?;
        let args = if self.match_token(TOKEN_PUNCTUATOR, Some("(")).is_some() {
            let a = self.parse_args()?;
            self.expect(TOKEN_PUNCTUATOR, Some(")"))?;
            a
        } else {
            Vec::new()
        };
        let init = if self.check(TOKEN_PUNCTUATOR, Some("{")) {
            Some(Box::new(self.parse_primary()?))
        } else {
            None
        };
        Ok(Expr::Novum {
            locus,
            callee: Box::new(callee),
            args,
            init,
        })
    }

    fn parse_finge(&mut self) -> Result<Expr, CompileError> {
        use crate::LitteraSpecies;

        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("finge"))?;
        let variant = self.expect(TOKEN_IDENTIFIER, None)?.valor;
        self.expect(TOKEN_PUNCTUATOR, Some("{"))?;

        let mut campi = Vec::new();
        if !self.check(TOKEN_PUNCTUATOR, Some("}")) {
            loop {
                let loc = self.peek(0).locus;
                let name = self.expect_name()?.valor;
                let key = Expr::Littera {
                    locus: loc,
                    species: LitteraSpecies::Textus,
                    valor: name,
                };
                self.expect(TOKEN_PUNCTUATOR, Some(":"))?;
                let valor = self.parse_expr(0)?;
                campi.push(ObiectumProp {
                    locus: loc,
                    key,
                    valor,
                    shorthand: false,
                    computed: false,
                });
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
        }
        self.expect(TOKEN_PUNCTUATOR, Some("}"))?;

        let typus = if self.match_token(TOKEN_KEYWORD, Some("qua")).is_some() {
            Some(self.parse_typus()?)
        } else {
            None
        };

        Ok(Expr::Finge {
            locus,
            variant,
            campi,
            typus,
        })
    }

    fn parse_clausura(&mut self) -> Result<Expr, CompileError> {
        use crate::ClausuraCorpus;

        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("clausura"))?;

        let mut params = Vec::new();
        if self.check(TOKEN_IDENTIFIER, None) {
            loop {
                let loc = self.peek(0).locus;
                let nomen = self.expect(TOKEN_IDENTIFIER, None)?.valor;
                let typus = if self.match_token(TOKEN_PUNCTUATOR, Some(":")).is_some() {
                    Some(self.parse_typus()?)
                } else {
                    None
                };
                params.push(Param {
                    locus: loc,
                    nomen,
                    typus,
                    default: None,
                    rest: false,
                });
                if self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_none() {
                    break;
                }
            }
        }

        let corpus = if self.check(TOKEN_PUNCTUATOR, Some("{")) {
            ClausuraCorpus::Stmt(Box::new(self.parse_massa()?))
        } else {
            self.expect(TOKEN_PUNCTUATOR, Some(":"))?;
            ClausuraCorpus::Expr(Box::new(self.parse_expr(0)?))
        };

        Ok(Expr::Clausura { locus, params, corpus })
    }

    fn parse_scriptum(&mut self) -> Result<Expr, CompileError> {
        let locus = self.peek(0).locus;
        self.expect(TOKEN_KEYWORD, Some("scriptum"))?;
        self.expect(TOKEN_PUNCTUATOR, Some("("))?;
        let template = self.expect(TOKEN_TEXTUS, None)?.valor;
        let mut args = Vec::new();
        while self.match_token(TOKEN_PUNCTUATOR, Some(",")).is_some() {
            args.push(self.parse_expr(0)?);
        }
        self.expect(TOKEN_PUNCTUATOR, Some(")"))?;
        Ok(Expr::Scriptum { locus, template, args })
    }
}

/// Filter out comments and newlines from token stream.
pub fn prepare(tokens: Vec<Token>) -> Vec<Token> {
    tokens
        .into_iter()
        .filter(|t| t.tag != crate::TOKEN_COMMENT && t.tag != crate::TOKEN_NEWLINE)
        .collect()
}

/// Parse tokens into a module.
pub fn parse(tokens: Vec<Token>, filename: &str) -> Result<Modulus, CompileError> {
    Parser::new(tokens, filename).parse()
}
