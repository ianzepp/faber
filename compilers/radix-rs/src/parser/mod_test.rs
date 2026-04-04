use super::parse;
use crate::lexer::lex;
use crate::syntax::{
    AnnotationKind, BindingPattern, ClassMemberKind, CuraKind, IfBody, ImportKind, InlineReturn, IteraMode,
    Mutability, ParamMode, Pattern, PatternBind, PraeparaKind, ProbaModifier, ScribeKind, SecusClause, StmtKind,
};

fn parse_program(source: &str) -> super::ParseResult {
    parse(lex(source))
}

fn parse_ok(source: &str) -> super::ParseResult {
    let result = parse_program(source);
    assert!(
        result.errors.is_empty(),
        "expected parse success, got errors: {:?}",
        result
            .errors
            .iter()
            .map(|err| err.message.as_str())
            .collect::<Vec<_>>()
    );
    result
}

fn symbol_name(result: &super::ParseResult, symbol: crate::lexer::Symbol) -> &str {
    result.interner.resolve(symbol)
}

#[test]
fn recovery_stops_at_transfer_statement_boundaries() {
    let result = parse_program(
        r#"
fixum = 1
rumpe
"#,
    );

    assert_eq!(result.errors.len(), 1, "expected one parse error");

    let program = result.program.expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1, "recovery should preserve the following statement");
    assert!(matches!(program.stmts[0].kind, StmtKind::Rumpe(_)));
}

#[test]
fn recovery_stops_at_block_closure() {
    let result = parse_program(
        r#"
incipit {
    fixum = 1
    rumpe
}
"#,
    );

    assert_eq!(result.errors.len(), 1, "expected one parse error");

    let program = result.program.expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1);

    let StmtKind::Incipit(entry) = &program.stmts[0].kind else {
        panic!("expected incipit statement");
    };

    let crate::syntax::IfBody::Block(block) = &entry.body else {
        panic!("expected block body");
    };

    assert_eq!(block.stmts.len(), 1, "recovery should preserve valid statements inside the block");
    assert!(matches!(block.stmts[0].kind, StmtKind::Rumpe(_)));
}

#[test]
fn recovery_preserves_annotated_following_statements() {
    let result = parse_program(
        r#"
fixum = 1
@ futura
functio vale() {}
"#,
    );

    assert_eq!(result.errors.len(), 1, "expected one parse error");

    let program = result.program.expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1, "recovery should preserve the following declaration");

    let StmtKind::Func(func) = &program.stmts[0].kind else {
        panic!("expected function declaration");
    };

    assert_eq!(program.stmts[0].annotations.len(), 1, "annotation should remain attached after recovery");
    assert!(func.body.is_some());
}

#[test]
fn parses_following_variable_after_if_body_error() {
    let result = parse_program(
        r#"
si verum x
fixum numerus postea = 2
"#,
    );

    assert_eq!(result.errors.len(), 1, "expected one parse error");

    let program = result.program.expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1, "recovery should preserve the following variable declaration");

    let StmtKind::Var(var) = &program.stmts[0].kind else {
        panic!("expected variable declaration");
    };

    let BindingPattern::Ident(name) = &var.binding else {
        panic!("expected identifier binding");
    };

    assert_eq!(result.interner.resolve(name.name), "postea");
}

#[test]
fn parses_declaration_keywords_and_shapes() {
    let result = parse_ok(
        r#"
§ opus nomen "demo"
importa ex "mod" publica Value ut Alias
fixum numerus count = 1
varia [head, ceteri tail] = values
@ futura
functio mitte(prae typus T, si de textus nomen ut alias vel "anon", ex numerus item, ceteri T reliqua) curata allocator errata Error exitus 7 immutata iacit optiones Opts -> textus {}
typus Nomen = textus
ordo Status { Bonus = 1, Malus = -1, Textus = "x" }
discretio Resultatus<T> { Ok { T valor }, Err { textus nuntius } }
"#,
    );

    assert_eq!(result.program.as_ref().expect("program").directives.len(), 1);

    let program = result.program.as_ref().expect("parser should produce a program");
    assert_eq!(program.stmts.len(), 7);

    let StmtKind::Import(import) = &program.stmts[0].kind else {
        panic!("expected import statement");
    };
    assert_eq!(symbol_name(&result, import.path), "mod");
    match &import.kind {
        ImportKind::Named { name, alias } => {
            assert_eq!(symbol_name(&result, name.name), "Value");
            assert_eq!(symbol_name(&result, alias.as_ref().expect("alias").name), "Alias");
        }
        _ => panic!("expected named import"),
    }

    let StmtKind::Var(const_var) = &program.stmts[1].kind else {
        panic!("expected const variable declaration");
    };
    assert!(matches!(const_var.binding, BindingPattern::Ident(_)));

    let StmtKind::Var(array_var) = &program.stmts[2].kind else {
        panic!("expected destructuring variable declaration");
    };
    let BindingPattern::Array { elements, rest, .. } = &array_var.binding else {
        panic!("expected array binding pattern");
    };
    assert_eq!(elements.len(), 1);
    assert_eq!(symbol_name(&result, rest.as_ref().expect("rest binding").name), "tail");

    let StmtKind::Func(func) = &program.stmts[3].kind else {
        panic!("expected function declaration");
    };
    assert_eq!(program.stmts[3].annotations.len(), 1);
    assert!(matches!(program.stmts[3].annotations[0].kind, AnnotationKind::Statement(_)));
    assert_eq!(func.type_params.len(), 1);
    assert_eq!(func.params.len(), 3);
    assert!(func.body.is_some());
    assert!(func.params[0].optional);
    assert!(matches!(func.params[0].mode, ParamMode::Ref));
    assert_eq!(symbol_name(&result, func.params[0].alias.as_ref().expect("alias").name), "alias");
    assert!(func.params[0].default.is_some());
    assert!(matches!(func.params[1].mode, ParamMode::Move));
    assert!(func.params[2].rest);
    assert_eq!(func.modifiers.len(), 6);

    let StmtKind::TypeAlias(alias) = &program.stmts[4].kind else {
        panic!("expected type alias");
    };
    assert_eq!(symbol_name(&result, alias.name.name), "Nomen");

    let StmtKind::Enum(enum_decl) = &program.stmts[5].kind else {
        panic!("expected enum declaration");
    };
    assert_eq!(enum_decl.members.len(), 3);

    let StmtKind::Union(union_decl) = &program.stmts[6].kind else {
        panic!("expected union declaration");
    };
    assert_eq!(union_decl.type_params.len(), 1);
    assert_eq!(union_decl.variants.len(), 2);
    assert_eq!(union_decl.variants[0].fields.len(), 1);
}

#[test]
fn parses_class_interface_and_test_keywords() {
    let result = parse_ok(
        r#"
abstractus genus Animal<T> sub Vivens implet Canens, Currens {
    @ futura
    functio canta(textus vox) -> vacuum {}
    generis textus nomen: "leo"
    nexum numerus aetas: 3
}
pactum Canens<T> {
    functio canta(textus vox) iacit -> vacuum
}
probandum "suite" {
    praepara omnia {}
    postpara {}
    proba omitte "skip" futurum "later" solum tag "focus" temporis 5 metior repete 2 fragilis 1 requirit "net" solum_in "ci" "case" {}
    probandum "nested" { proba "inner" {} }
}
"#,
    );

    let program = result.program.as_ref().expect("parser should produce a program");
    assert_eq!(program.stmts.len(), 3);

    let StmtKind::Class(class) = &program.stmts[0].kind else {
        panic!("expected class declaration");
    };
    assert!(class.is_abstract);
    assert!(class.extends.is_some());
    assert_eq!(class.implements.len(), 2);
    assert_eq!(class.members.len(), 3);
    assert!(matches!(class.members[0].kind, ClassMemberKind::Method(_)));
    assert!(matches!(class.members[1].kind, ClassMemberKind::Field(_)));
    assert!(matches!(class.members[2].kind, ClassMemberKind::Field(_)));
    assert_eq!(class.members[0].annotations.len(), 1);

    let ClassMemberKind::Field(static_field) = &class.members[1].kind else {
        panic!("expected static field");
    };
    assert!(static_field.is_static);

    let ClassMemberKind::Field(bound_field) = &class.members[2].kind else {
        panic!("expected bound field");
    };
    assert!(bound_field.is_bound);

    let StmtKind::Interface(interface) = &program.stmts[1].kind else {
        panic!("expected interface declaration");
    };
    assert_eq!(interface.type_params.len(), 1);
    assert_eq!(interface.methods.len(), 1);
    assert_eq!(interface.methods[0].modifiers.len(), 1);

    let StmtKind::Probandum(test) = &program.stmts[2].kind else {
        panic!("expected probandum declaration");
    };
    assert_eq!(test.body.setup.len(), 2);
    assert!(matches!(test.body.setup[0].kind, PraeparaKind::Praepara));
    assert!(test.body.setup[0].all);
    assert!(matches!(test.body.setup[1].kind, PraeparaKind::Postpara));
    assert_eq!(test.body.tests.len(), 1);
    assert_eq!(test.body.nested.len(), 1);

    let case = &test.body.tests[0];
    assert_eq!(case.modifiers.len(), 10);
    assert!(matches!(case.modifiers[0], ProbaModifier::Omitte(_)));
    assert!(matches!(case.modifiers[1], ProbaModifier::Futurum(_)));
    assert!(matches!(case.modifiers[2], ProbaModifier::Solum));
    assert!(matches!(case.modifiers[3], ProbaModifier::Tag(_)));
    assert!(matches!(case.modifiers[4], ProbaModifier::Temporis(5)));
    assert!(matches!(case.modifiers[5], ProbaModifier::Metior));
    assert!(matches!(case.modifiers[6], ProbaModifier::Repete(2)));
    assert!(matches!(case.modifiers[7], ProbaModifier::Fragilis(1)));
    assert!(matches!(case.modifiers[8], ProbaModifier::Requirit(_) | ProbaModifier::SolumIn(_)));
    assert!(
        case.modifiers.iter().any(|modifier| matches!(modifier, ProbaModifier::Requirit(_))),
        "expected requirit modifier"
    );
    assert!(
        case.modifiers.iter().any(|modifier| matches!(modifier, ProbaModifier::SolumIn(_))),
        "expected solumIn modifier"
    );
}

#[test]
fn parses_control_flow_transfer_and_clause_keywords() {
    let result = parse_ok(
        r#"
si verum ergo scribe "yes" cape err {} sin falsum reddit 1 secus tacet
dum verum { perge } cape err {}
itera pro items fixum item reddit item cape err {}
elige value { casu 1 reddit 1 ceterum tacet } cape err {}
discerne omnia value, other { casu Ok ut result, _ reddit result ceterum moritor "bad" }
custodi { si verum reddit 1 si falsum ergo scribe "no" }
fac {} cape err {} dum verum
tempta {} cape err {} demum {}
adfirma verum, "msg"
scribe "a", "b"
vide "c"
mone "d"
redde 1
rumpe
perge
iace "err"
mori "panic"
"#,
    );

    let program = result.program.as_ref().expect("parser should produce a program");
    assert_eq!(program.stmts.len(), 17);

    let StmtKind::Si(if_stmt) = &program.stmts[0].kind else {
        panic!("expected if statement");
    };
    assert!(matches!(if_stmt.then, IfBody::Ergo(_)));
    assert!(if_stmt.catch.is_some());
    let SecusClause::Sin(sin_stmt) = if_stmt.else_.as_ref().expect("else clause") else {
        panic!("expected sin clause");
    };
    assert!(matches!(sin_stmt.then, IfBody::InlineReturn(InlineReturn::Reddit(_))));
    assert!(matches!(sin_stmt.else_.as_ref().expect("secus clause"), SecusClause::InlineReturn(InlineReturn::Tacet)));

    let StmtKind::Dum(while_stmt) = &program.stmts[1].kind else {
        panic!("expected while statement");
    };
    assert!(matches!(while_stmt.body, IfBody::Block(_)));
    assert!(while_stmt.catch.is_some());

    let StmtKind::Itera(iter_stmt) = &program.stmts[2].kind else {
        panic!("expected itera statement");
    };
    assert!(matches!(iter_stmt.mode, IteraMode::Pro));
    assert!(matches!(iter_stmt.mutability, Mutability::Immutable));
    assert!(matches!(iter_stmt.body, IfBody::InlineReturn(InlineReturn::Reddit(_))));

    let StmtKind::Elige(switch_stmt) = &program.stmts[3].kind else {
        panic!("expected elige statement");
    };
    assert_eq!(switch_stmt.cases.len(), 1);
    assert!(switch_stmt.default.is_some());
    assert!(switch_stmt.catch.is_some());

    let StmtKind::Discerne(match_stmt) = &program.stmts[4].kind else {
        panic!("expected discerne statement");
    };
    assert!(match_stmt.exhaustive);
    assert_eq!(match_stmt.subjects.len(), 2);
    assert_eq!(match_stmt.arms.len(), 1);
    assert!(match_stmt.default.is_some());
    match &match_stmt.arms[0].patterns[0] {
        Pattern::Ident(_, Some(PatternBind::Alias(alias))) => {
            assert_eq!(symbol_name(&result, alias.name), "result");
        }
        _ => panic!("expected alias-bound pattern"),
    }

    let StmtKind::Custodi(guard_stmt) = &program.stmts[5].kind else {
        panic!("expected custodi statement");
    };
    assert_eq!(guard_stmt.clauses.len(), 2);

    let StmtKind::Fac(fac_stmt) = &program.stmts[6].kind else {
        panic!("expected fac statement");
    };
    assert!(fac_stmt.catch.is_some());
    assert!(fac_stmt.while_.is_some());

    let StmtKind::Tempta(try_stmt) = &program.stmts[7].kind else {
        panic!("expected tempta statement");
    };
    assert!(try_stmt.catch.is_some());
    assert!(try_stmt.finally.is_some());

    assert!(matches!(program.stmts[8].kind, StmtKind::Adfirma(_)));
    match &program.stmts[9].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Scribe)),
        _ => panic!("expected scribe statement"),
    }
    match &program.stmts[10].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Vide)),
        _ => panic!("expected vide statement"),
    }
    match &program.stmts[11].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Mone)),
        _ => panic!("expected mone statement"),
    }
    assert!(matches!(program.stmts[12].kind, StmtKind::Redde(_)));
    assert!(matches!(program.stmts[13].kind, StmtKind::Rumpe(_)));
    assert!(matches!(program.stmts[14].kind, StmtKind::Perge(_)));
    assert!(matches!(program.stmts[15].kind, StmtKind::Iace(_)));
    assert!(matches!(program.stmts[16].kind, StmtKind::Mori(_)));
}

#[test]
fn parses_entry_resource_endpoint_and_extract_keywords() {
    let result = parse_ok(
        r#"
incipit argumenta args exitus 1 {}
incipiet argumenta argv reddit argv
cura arena {}
cura page source fixum textus page {}
ad "/salve" (request, sparge extra) -> textus pro res ut alias {} cape err {}
ex source varia nomen ut name, ceteri reliqua
"#,
    );

    let program = result.program.as_ref().expect("parser should produce a program");
    assert_eq!(program.stmts.len(), 6);

    let StmtKind::Incipit(main_stmt) = &program.stmts[0].kind else {
        panic!("expected incipit statement");
    };
    assert!(!main_stmt.is_async);
    assert!(main_stmt.args.is_some());
    assert!(main_stmt.exitus.is_some());
    assert!(matches!(main_stmt.body, IfBody::Block(_)));

    let StmtKind::Incipit(async_main) = &program.stmts[1].kind else {
        panic!("expected incipiet statement");
    };
    assert!(async_main.is_async);
    assert!(matches!(async_main.body, IfBody::InlineReturn(InlineReturn::Reddit(_))));

    let StmtKind::Cura(arena_stmt) = &program.stmts[2].kind else {
        panic!("expected anonymous arena cura");
    };
    assert!(matches!(arena_stmt.kind, Some(CuraKind::Arena)));
    assert!(arena_stmt.init.is_none());

    let StmtKind::Cura(page_stmt) = &program.stmts[3].kind else {
        panic!("expected bound page cura");
    };
    assert!(matches!(page_stmt.kind, Some(CuraKind::Page)));
    assert!(page_stmt.init.is_some());
    assert!(matches!(page_stmt.mutability, Mutability::Immutable));

    let StmtKind::Ad(endpoint) = &program.stmts[4].kind else {
        panic!("expected ad statement");
    };
    assert_eq!(symbol_name(&result, endpoint.path), "/salve");
    assert_eq!(endpoint.args.len(), 2);
    assert!(endpoint.args[1].spread);
    let binding = endpoint.binding.as_ref().expect("endpoint binding");
    assert_eq!(symbol_name(&result, binding.name.name), "res");
    assert_eq!(symbol_name(&result, binding.alias.as_ref().expect("alias").name), "alias");
    assert!(endpoint.body.is_some());
    assert!(endpoint.catch.is_some());

    let StmtKind::Ex(extract) = &program.stmts[5].kind else {
        panic!("expected ex destructuring statement");
    };
    assert!(matches!(extract.mutability, Mutability::Mutable));
    assert_eq!(extract.fields.len(), 1);
    assert_eq!(symbol_name(&result, extract.fields[0].alias.as_ref().expect("field alias").name), "name");
    assert_eq!(symbol_name(&result, extract.rest.as_ref().expect("rest binding").name), "reliqua");
}
