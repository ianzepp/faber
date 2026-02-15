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
