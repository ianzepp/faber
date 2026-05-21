use super::lower;
use crate::hir::{HirFunction, HirItemKind, HirTestMetadata, HirTestModifier};
use crate::lexer::Interner;
use crate::semantic::{passes, Resolver, TypeTable};

fn lower_source(source: &str) -> (crate::hir::HirProgram, Interner) {
    let lex_result = crate::lexer::lex(source);
    let parse_result = crate::parser::parse(lex_result);
    assert!(
        parse_result.success(),
        "parse failed: {:?}",
        parse_result
            .errors
            .iter()
            .map(|error| &error.message)
            .collect::<Vec<_>>()
    );

    let program = parse_result.program.expect("expected parsed program");
    let interner = parse_result.interner;
    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();

    if let Err(errors) = passes::collect::collect(&program, &mut resolver, &mut types) {
        panic!("collect failed: {:?}", errors);
    }
    if let Err(errors) = passes::resolve::resolve(&program, &mut resolver, &interner, &mut types) {
        panic!("resolve failed: {:?}", errors);
    }

    let (hir, errors) = lower(&program, &resolver, &mut types, &interner);
    assert!(errors.is_empty(), "lower failed: {:?}", errors);
    (hir, interner)
}

fn test_metadata(item: &crate::hir::HirItem) -> &HirTestMetadata {
    let HirItemKind::Function(HirFunction { test: Some(test), .. }) = &item.kind else {
        panic!("expected lowered test function");
    };
    test
}

#[test]
fn lowers_suite_paths_for_top_level_and_nested_tests() {
    let source = r#"proba "top level case" {
  adfirma verum
}

probandum "outer suite" {
  proba "nested case" {
    adfirma verum
  }

  probandum "inner suite" {
    proba "deep case" {
      adfirma verum
    }
  }
}"#;

    let (hir, interner) = lower_source(source);

    assert_eq!(hir.items.len(), 3);

    let top = test_metadata(&hir.items[0]);
    assert_eq!(interner.resolve(top.name), "top level case");
    assert!(top.suite_path.is_empty());
    assert_eq!(top.span, hir.items[0].span);
    assert!(top.modifiers.is_empty());

    let nested = test_metadata(&hir.items[1]);
    assert_eq!(interner.resolve(nested.name), "nested case");
    assert_eq!(nested.suite_path.len(), 1);
    assert_eq!(interner.resolve(nested.suite_path[0]), "outer suite");
    assert_eq!(nested.span, hir.items[1].span);
    assert!(nested.modifiers.is_empty());

    let deep = test_metadata(&hir.items[2]);
    assert_eq!(interner.resolve(deep.name), "deep case");
    assert_eq!(deep.suite_path.len(), 2);
    assert_eq!(interner.resolve(deep.suite_path[0]), "outer suite");
    assert_eq!(interner.resolve(deep.suite_path[1]), "inner suite");
    assert_eq!(deep.span, hir.items[2].span);
    assert!(deep.modifiers.is_empty());
}

#[test]
fn lowers_all_test_modifiers_into_metadata() {
    let source = r#"proba "modifier case" omitte "blocked by service" futurum "later" solum tag "smoke" temporis 5 metior repete 2 fragilis 1 requirit "capability" solum_in "staging" {
    adfirma verum
}"#;

    let (hir, interner) = lower_source(source);
    assert_eq!(hir.items.len(), 1);

    let metadata = test_metadata(&hir.items[0]);
    assert_eq!(interner.resolve(metadata.name), "modifier case");
    assert!(metadata.suite_path.is_empty());
    assert_eq!(metadata.span, hir.items[0].span);

    let rendered: Vec<String> = metadata
        .modifiers
        .iter()
        .map(|modifier| match modifier {
            HirTestModifier::Omitte(reason) => format!("omitte({})", interner.resolve(*reason)),
            HirTestModifier::Futurum(reason) => format!("futurum({})", interner.resolve(*reason)),
            HirTestModifier::Solum => "solum".to_owned(),
            HirTestModifier::Tag(tag) => format!("tag({})", interner.resolve(*tag)),
            HirTestModifier::Temporis(n) => format!("temporis({n})"),
            HirTestModifier::Metior => "metior".to_owned(),
            HirTestModifier::Repete(n) => format!("repete({n})"),
            HirTestModifier::Fragilis(n) => format!("fragilis({n})"),
            HirTestModifier::Requirit(req) => format!("requirit({})", interner.resolve(*req)),
            HirTestModifier::SolumIn(env) => format!("solum_in({})", interner.resolve(*env)),
        })
        .collect();

    assert_eq!(
        rendered,
        vec![
            "omitte(blocked by service)".to_owned(),
            "futurum(later)".to_owned(),
            "solum".to_owned(),
            "tag(smoke)".to_owned(),
            "temporis(5)".to_owned(),
            "metior".to_owned(),
            "repete(2)".to_owned(),
            "fragilis(1)".to_owned(),
            "requirit(capability)".to_owned(),
            "solum_in(staging)".to_owned(),
        ]
    );
}

#[test]
fn lowers_suite_modifiers_into_nested_test_metadata() {
    let source = r#"probandum "suite" tag "parser" solum {
  proba "case" tag "unit" {
    adfirma verum
  }
}"#;

    let (hir, interner) = lower_source(source);
    assert_eq!(hir.items.len(), 1);

    let metadata = test_metadata(&hir.items[0]);
    let rendered: Vec<String> = metadata
        .modifiers
        .iter()
        .map(|modifier| match modifier {
            HirTestModifier::Tag(tag) => format!("tag({})", interner.resolve(*tag)),
            HirTestModifier::Solum => "solum".to_owned(),
            other => format!("{other:?}"),
        })
        .collect();

    assert_eq!(
        rendered,
        vec!["tag(parser)".to_owned(), "solum".to_owned(), "tag(unit)".to_owned()]
    );
}
