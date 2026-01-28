use std::collections::HashSet;
use subsidia_rs::{CompileError, Locus, Token};

/// Tokenize Faber source code.
pub fn lex(source: &str, filename: &str) -> Result<Vec<Token>, CompileError> {
    let mut lexer = Lexer::new(source, filename);
    lexer.lex()
}

struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    linea: i32,
    line_start: usize,
    filename: String,
    keywords: HashSet<&'static str>,
    punctuators: HashSet<char>,
    operators: Vec<&'static str>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, filename: &str) -> Self {
        let keywords: HashSet<&str> = [
            // Declarations
            "varia",
            "fixum",
            "functio",
            "genus",
            "pactum",
            "ordo",
            "discretio",
            "ex",
            "importa",
            "ut",
            // Modifiers
            "publica",
            "privata",
            "protecta",
            "generis",
            "implet",
            "sub",
            // Control flow
            "si",
            "sin",
            "secus",
            "dum",
            "fac",
            "elige",
            "casu",
            "ceterum",
            "discerne",
            "custodi",
            "de",
            "itera",
            "in",
            "pro",
            "omnia",
            // Actions
            "redde",
            "reddit",
            "rumpe",
            "perge",
            "iace",
            "mori",
            "tempta",
            "cape",
            "demum",
            "scribe",
            "vide",
            "mone",
            "adfirma",
            "tacet",
            // Expressions
            "cede",
            "novum",
            "clausura",
            "qua",
            "innatum",
            "finge",
            "sic",
            "scriptum",
            // Operators (word-form)
            "et",
            "aut",
            "vel",
            "inter",
            "intra",
            "non",
            "nihil",
            "nonnihil",
            "positivum",
            // Literals
            "verum",
            "falsum",
            "ego",
            // Entry
            "incipit",
            "incipiet",
            // Test
            "probandum",
            "proba",
            // Type
            "usque",
            // Annotations
            "publicum",
            "externa",
            // Body shortcuts
            "ergo",
            "tacet",
            "iacit",
            "moritor",
            // Iteration
            "ceteri",
        ]
        .into_iter()
        .collect();

        let punctuators: HashSet<char> = [
            '(', ')', '{', '}', '[', ']', ',', '.', ':', ';', '@', '#', '?', '!',
        ]
        .into_iter()
        .collect();

        let operators = vec![
            // Multi-char first (greedy match)
            "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??", "+=", "-=", "*=", "/=", "->",
            "..", // Single-char
            "+", "-", "*", "/", "%", "<", ">", "=", "&", "|", "^", "~",
        ];

        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            linea: 1,
            line_start: 0,
            filename: filename.to_string(),
            keywords,
            punctuators,
            operators,
        }
    }

    fn locus(&self) -> Locus {
        Locus {
            linea: self.linea,
            columna: (self.pos - self.line_start + 1) as i32,
            index: self.pos as i32,
        }
    }

    fn peek(&self, offset: usize) -> u8 {
        let idx = self.pos + offset;
        if idx >= self.bytes.len() {
            0
        } else {
            self.bytes[idx]
        }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.bytes[self.pos];
        self.pos += 1;
        if ch == b'\n' {
            self.linea += 1;
            self.line_start = self.pos;
        }
        ch
    }

    fn match_str(&mut self, s: &str) -> bool {
        if self.source[self.pos..].starts_with(s) {
            for _ in 0..s.len() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    fn lex(&mut self) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();

        while self.pos < self.bytes.len() {
            self.skip_whitespace(&mut tokens);
            if self.pos >= self.bytes.len() {
                break;
            }

            let loc = self.locus();
            let ch = self.peek(0);

            // Comment
            if ch == b'#' {
                let value = self.read_comment();
                tokens.push(Token {
                    tag: "Comment".to_string(),
                    valor: value,
                    locus: loc,
                });
                continue;
            }

            // Triple-quoted string
            if ch == b'"' && self.peek(1) == b'"' && self.peek(2) == b'"' {
                let value = self.read_triple_string();
                tokens.push(Token {
                    tag: "Textus".to_string(),
                    valor: value,
                    locus: loc,
                });
                continue;
            }

            // String
            if ch == b'"' || ch == b'\'' {
                let value = self.read_string(ch);
                tokens.push(Token {
                    tag: "Textus".to_string(),
                    valor: value,
                    locus: loc,
                });
                continue;
            }

            // Number
            if is_digit(ch) {
                let value = self.read_number();
                tokens.push(Token {
                    tag: "Numerus".to_string(),
                    valor: value,
                    locus: loc,
                });
                continue;
            }

            // Identifier or keyword
            if is_alpha(ch) || ch == b'_' {
                let value = self.read_identifier();
                let tag = if self.keywords.contains(value.as_str()) {
                    "Keyword"
                } else {
                    "Identifier"
                };
                tokens.push(Token {
                    tag: tag.to_string(),
                    valor: value,
                    locus: loc,
                });
                continue;
            }

            // Operators (greedy match)
            let mut matched = false;
            for op in &self.operators.clone() {
                if self.match_str(op) {
                    tokens.push(Token {
                        tag: "Operator".to_string(),
                        valor: op.to_string(),
                        locus: loc,
                    });
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }

            // Section marker (UTF-8 multi-byte)
            if self.match_str("§") {
                tokens.push(Token {
                    tag: "Punctuator".to_string(),
                    valor: "§".to_string(),
                    locus: loc,
                });
                continue;
            }

            // Punctuators
            if self.punctuators.contains(&(ch as char)) {
                self.advance();
                tokens.push(Token {
                    tag: "Punctuator".to_string(),
                    valor: (ch as char).to_string(),
                    locus: loc,
                });
                continue;
            }

            return Err(CompileError::new(
                format!("unexpected character '{}'", ch as char),
                loc,
                &self.filename,
            ));
        }

        tokens.push(Token {
            tag: "EOF".to_string(),
            valor: String::new(),
            locus: self.locus(),
        });

        Ok(tokens)
    }

    fn skip_whitespace(&mut self, tokens: &mut Vec<Token>) {
        while self.pos < self.bytes.len() {
            let ch = self.peek(0);
            if ch == b' ' || ch == b'\t' || ch == b'\r' {
                self.advance();
            } else if ch == b'\n' {
                let loc = self.locus();
                self.advance();
                tokens.push(Token {
                    tag: "Newline".to_string(),
                    valor: "\n".to_string(),
                    locus: loc,
                });
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: u8) -> String {
        let mut result = String::new();
        self.advance(); // skip opening quote

        while self.pos < self.bytes.len() && self.peek(0) != quote {
            if self.peek(0) == b'\\' {
                self.advance();
                let esc = self.advance();
                match esc {
                    b'n' => result.push('\n'),
                    b't' => result.push('\t'),
                    b'r' => result.push('\r'),
                    b'\\' => result.push('\\'),
                    b'"' => result.push('"'),
                    b'\'' => result.push('\''),
                    _ => result.push(esc as char),
                }
            } else {
                // Handle UTF-8 properly by reading chars from source slice
                let remaining = &self.source[self.pos..];
                if let Some(ch) = remaining.chars().next() {
                    result.push(ch);
                    let byte_len = ch.len_utf8();
                    for _ in 0..byte_len {
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }
        }
        if self.pos < self.bytes.len() {
            self.advance(); // skip closing quote
        }
        result
    }

    fn read_triple_string(&mut self) -> String {
        // Skip opening """
        self.advance();
        self.advance();
        self.advance();

        // Skip leading newline if present
        if self.peek(0) == b'\n' {
            self.advance();
        }

        let mut result = String::new();
        while self.pos < self.bytes.len() {
            if self.peek(0) == b'"' && self.peek(1) == b'"' && self.peek(2) == b'"' {
                // Trim trailing newline
                if result.ends_with('\n') {
                    result.pop();
                }
                self.advance();
                self.advance();
                self.advance();
                return result;
            }
            // Handle UTF-8 properly
            let remaining = &self.source[self.pos..];
            if let Some(ch) = remaining.chars().next() {
                result.push(ch);
                let byte_len = ch.len_utf8();
                for _ in 0..byte_len {
                    self.advance();
                }
            } else {
                self.advance();
            }
        }
        result
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();
        while self.pos < self.bytes.len() && is_number_char(self.peek(0)) {
            result.push(self.advance() as char);
        }
        result
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while self.pos < self.bytes.len() && is_ident_char(self.peek(0)) {
            result.push(self.advance() as char);
        }
        result
    }

    fn read_comment(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // skip #
        while self.pos < self.bytes.len() && self.peek(0) != b'\n' {
            result.push(self.advance() as char);
        }
        result
    }
}

fn is_digit(ch: u8) -> bool {
    ch >= b'0' && ch <= b'9'
}

fn is_alpha(ch: u8) -> bool {
    (ch >= b'a' && ch <= b'z') || (ch >= b'A' && ch <= b'Z')
}

fn is_ident_char(ch: u8) -> bool {
    is_alpha(ch) || is_digit(ch) || ch == b'_'
}

fn is_number_char(ch: u8) -> bool {
    is_digit(ch) || ch == b'.' || ch == b'_'
}
