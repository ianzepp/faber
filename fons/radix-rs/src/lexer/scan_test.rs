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
    let result = lex("+ += - -= -> → * *= / /= % ^ == ≡ === != ≠ !== !. ![ !( < <= ≤ > >= ≥ && &= || |= ?. ?[ ?( ?? = ←");
    assert!(result.errors.is_empty());

    let kinds: Vec<TokenKind> = result.tokens.into_iter().map(|token| token.kind).collect();
    let expected = vec![
        TokenKind::Plus,
        TokenKind::PlusEq,
        TokenKind::Minus,
        TokenKind::MinusEq,
        TokenKind::Arrow,
        TokenKind::Arrow,
        TokenKind::Star,
        TokenKind::StarEq,
        TokenKind::Slash,
        TokenKind::SlashEq,
        TokenKind::Percent,
        TokenKind::Caret,
        TokenKind::EqEq,
        TokenKind::EqEq,
        TokenKind::EqEqEq,
        TokenKind::BangEq,
        TokenKind::BangEq,
        TokenKind::BangEqEq,
        TokenKind::BangDot,
        TokenKind::BangBracket,
        TokenKind::BangParen,
        TokenKind::Lt,
        TokenKind::LtEq,
        TokenKind::LtEq,
        TokenKind::Gt,
        TokenKind::GtEq,
        TokenKind::GtEq,
        TokenKind::AmpAmp,
        TokenKind::AmpEq,
        TokenKind::PipePipe,
        TokenKind::PipeEq,
        TokenKind::QuestionDot,
        TokenKind::QuestionBracket,
        TokenKind::QuestionParen,
        TokenKind::Question,
        TokenKind::Question,
        TokenKind::Eq,
        TokenKind::Eq,
        TokenKind::Eof,
    ];

    assert_eq!(kinds, expected);
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
