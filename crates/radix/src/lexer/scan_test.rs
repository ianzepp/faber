use super::Interner;
use crate::lexer::{lex, LexErrorKind, TokenKind};

#[test]
fn lexes_unicode_identifiers() {
    let result = lex("fixum 变量 = 1");
    assert!(result.errors.is_empty());
    assert!(result
        .tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Ident(_))));
}

#[test]
fn rejects_non_xid_identifier_start() {
    let result = lex("\u{0301}abc");
    assert!(!result.errors.is_empty());
    assert!(result
        .errors
        .iter()
        .any(|err| err.kind == LexErrorKind::UnexpectedCharacter));
}

#[test]
fn normalizes_interned_identifiers_to_nfc() {
    let mut interner = Interner::new();
    let composed = interner.intern("café");
    let decomposed = interner.intern("cafe\u{301}");

    assert_eq!(composed, decomposed);
    assert_eq!(interner.resolve(composed), "café");
}

#[test]
fn lexer_interns_equivalent_unicode_forms_as_one_symbol() {
    let result = lex("fixum café = 1\nfixum cafe\u{301} = 2");
    assert!(result.errors.is_empty());

    let mut idents = result.tokens.iter().filter_map(|token| match token.kind {
        TokenKind::Ident(sym) => Some(sym),
        _ => None,
    });

    let first = idents.next().expect("first identifier");
    let second = idents.next().expect("second identifier");
    assert_eq!(first, second);
}

#[test]
fn lexes_operator_tokens_consistently() {
    // Post-clean-break: only canonical glyphs produce compound tokens.
    // Old ASCII multi-char forms (== != <= >= -> += etc) are rejected at lex time.
    let result = lex("+ ⊕ - ⊖ * ⊛ / ⊘ % ⊻ ≡ ≠ ≤ ≥ → ¬ !. ![ !( < ≤ > ≥ ∧ ⊜ ∨ ⊚ ≪ ≫ ⇢ ?. ?[ ?( ?? = ←");
    assert!(result.errors.is_empty());

    let kinds: Vec<TokenKind> = result.tokens.into_iter().map(|token| token.kind).collect();
    let expected = vec![
        TokenKind::Plus,
        TokenKind::PlusEq,
        TokenKind::Minus,
        TokenKind::MinusEq,
        TokenKind::Star,
        TokenKind::StarEq,
        TokenKind::Slash,
        TokenKind::SlashEq,
        TokenKind::Percent,
        TokenKind::Caret,
        TokenKind::EqEq,
        TokenKind::BangEq,
        TokenKind::LtEq,
        TokenKind::GtEq,
        TokenKind::Arrow,
        TokenKind::Tilde,
        TokenKind::BangDot,
        TokenKind::BangBracket,
        TokenKind::BangParen,
        TokenKind::Lt,
        TokenKind::LtEq,
        TokenKind::Gt,
        TokenKind::GtEq,
        TokenKind::Amp,
        TokenKind::AmpEq,
        TokenKind::Pipe,
        TokenKind::PipeEq,
        TokenKind::Sinistratum,
        TokenKind::Dextratum,
        TokenKind::Verte,
        TokenKind::QuestionDot,
        TokenKind::QuestionBracket,
        TokenKind::QuestionParen,
        TokenKind::Question,
        TokenKind::Question,
        TokenKind::Eq,
        TokenKind::Assign,
        TokenKind::Eof,
    ];

    assert_eq!(kinds, expected);
}

#[test]
fn rejects_old_ascii_compound_operators() {
    // Explicit negative tests proving the clean break.
    // Old ASCII compounds no longer produce the compound TokenKind (they lex as separate chars).
    // Full parse will fail or misinterpret; here we prove lexer no longer recognizes them as units.
    let cases: &[(&str, TokenKind)] = &[
        ("==", TokenKind::EqEq),
        ("!=", TokenKind::BangEq),
        ("<=", TokenKind::LtEq),
        (">=", TokenKind::GtEq),
        ("->", TokenKind::Arrow),
        ("+=", TokenKind::PlusEq),
        ("-=", TokenKind::MinusEq),
        ("*=", TokenKind::StarEq),
        ("/=", TokenKind::SlashEq),
    ];
    for &(op, ref compound) in cases {
        let src = format!("fixum x = 1 {} 2", op);
        let result = lex(&src);
        // Lex itself does not error (single-char tokens are valid), but must not emit the compound.
        let emits_compound = result
            .tokens
            .iter()
            .any(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(compound));
        assert!(
            !emits_compound,
            "legacy operator {op} must not produce {compound:?} token after clean break",
        );
    }

    // Triple forms also gone (no EqEqEq etc from source).
    let triples = lex("1 === 2");
    assert!(!triples
        .tokens
        .iter()
        .any(|t| matches!(t.kind, TokenKind::EqEqEq | TokenKind::BangEqEq)));
}

#[test]
fn lexes_radix_numbers_and_reports_invalid_literals() {
    let result = lex("0xCAFE_F00D 0b1010_1111 0o755 0x");
    let kinds: Vec<TokenKind> = result
        .tokens
        .iter()
        .map(|token| token.kind.clone())
        .collect();

    assert_eq!(kinds[0], TokenKind::Integer(3_405_705_229));
    assert_eq!(kinds[1], TokenKind::Integer(175));
    assert_eq!(kinds[2], TokenKind::Integer(493));
    assert_eq!(kinds[3], TokenKind::Eof);
    assert!(result
        .errors
        .iter()
        .any(|err| err.kind == LexErrorKind::InvalidNumber && err.message == "invalid hexadecimal number"));
}

#[test]
fn lexes_unicode_range_operators() {
    let result = lex("0‥5 0…5 0 ante 5 0 usque 5");
    assert!(result.errors.is_empty());

    let kinds: Vec<TokenKind> = result.tokens.into_iter().map(|token| token.kind).collect();
    let expected = vec![
        TokenKind::Integer(0),
        TokenKind::DotDot,
        TokenKind::Integer(5),
        TokenKind::Integer(0),
        TokenKind::Ellipsis,
        TokenKind::Integer(5),
        TokenKind::Integer(0),
        TokenKind::Ante,
        TokenKind::Integer(5),
        TokenKind::Integer(0),
        TokenKind::Usque,
        TokenKind::Integer(5),
        TokenKind::Eof,
    ];

    assert_eq!(kinds, expected);
}

#[test]
fn lexes_conversio_glyph() {
    let result = lex("x ⇒ numerus");
    assert!(result.errors.is_empty());
    assert!(
        matches!(result.tokens[1].kind, TokenKind::Conversio),
        "'⇒' should lex as TokenKind::Conversio"
    );
}

#[test]
fn removed_verte_aliases_now_lex_as_identifiers() {
    // Post verte-alias-clean-break: qua/innatum/novum are ordinary identifiers in normal mode.
    // Only the ⇢ glyph produces TokenKind::Verte.
    for word in &["qua", "innatum", "novum"] {
        let result = lex(&format!("x {} textus", word));
        assert!(result.errors.is_empty(), "failed for word: {}", word);
        assert!(
            matches!(&result.tokens[1].kind, TokenKind::Ident(_)),
            "'{}' should now lex as Ident (Verte aliases removed)",
            word
        );
    }
}

#[test]
fn lexes_proba_modifier_keywords() {
    let result = lex(
        r#"proba "case" omitte "blocked" futurum "later" solum tag "focus" temporis 5 metior repete 2 fragilis 1 requirit "net" solum_in "ci" {}"#,
    );
    assert!(result.errors.is_empty());

    let kinds: Vec<TokenKind> = result.tokens.into_iter().map(|token| token.kind).collect();

    assert!(matches!(kinds[0], TokenKind::Proba));
    assert!(matches!(kinds[1], TokenKind::String(_)));
    assert!(matches!(kinds[2], TokenKind::Omitte));
    assert!(matches!(kinds[3], TokenKind::String(_)));
    assert!(matches!(kinds[4], TokenKind::Futurum));
    assert!(matches!(kinds[5], TokenKind::String(_)));
    assert!(matches!(kinds[6], TokenKind::Solum));
    assert!(matches!(kinds[7], TokenKind::Tag));
    assert!(matches!(kinds[8], TokenKind::String(_)));
    assert!(matches!(kinds[9], TokenKind::Temporis));
    assert!(matches!(kinds[10], TokenKind::Integer(5)));
    assert!(matches!(kinds[11], TokenKind::Metior));
    assert!(matches!(kinds[12], TokenKind::Repete));
    assert!(matches!(kinds[13], TokenKind::Integer(2)));
    assert!(matches!(kinds[14], TokenKind::Fragilis));
    assert!(matches!(kinds[15], TokenKind::Integer(1)));
    assert!(matches!(kinds[16], TokenKind::Requirit));
    assert!(matches!(kinds[17], TokenKind::String(_)));
    assert!(matches!(kinds[18], TokenKind::SolumIn));
    assert!(matches!(kinds[19], TokenKind::String(_)));
    assert!(matches!(kinds[20], TokenKind::LBrace));
    assert!(matches!(kinds[21], TokenKind::RBrace));
    assert!(matches!(kinds[22], TokenKind::Eof));
}
