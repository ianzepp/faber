use super::parse;
use crate::lexer::lex;
use crate::syntax::{
    AnnotationKind, BindingPattern, ClassMemberKind, ClausuraBody, CuraKind, ExprKind, IfBody, ImportKind, IteraMode,
    Literal, Mutability, ParamMode, Pattern, PatternBind, PraeparaKind, ProbaModifier, ScribeKind, SecusClause,
    StmtKind, TypeExprKind,
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

fn assert_parse_error_contains(source: &str, expected: &str) {
    let result = parse_program(source);
    assert!(
        result
            .errors
            .iter()
            .any(|err| err.message.contains(expected)),
        "expected parse error containing {expected:?}, got {:?}",
        result
            .errors
            .iter()
            .map(|err| err.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn parses_cli_and_imperium_annotations_as_structured_ast() {
    let result = parse_ok(
        r#"
@ cli "faber"
@ versio "0.1.0"
incipit argumenta args {}

@ imperium "emit"
@ alias "e"
functio emit() {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    assert_eq!(program.stmts.len(), 2);

    assert!(matches!(
        &program.stmts[0].annotations[0].kind,
        AnnotationKind::Cli(cli) if symbol_name(&result, cli.name) == "faber"
    ));
    assert!(
        matches!(program.stmts[0].annotations[1].kind, AnnotationKind::Statement(_)),
        "@ versio remains generic in phase 01"
    );

    assert!(matches!(
        &program.stmts[1].annotations[0].kind,
        AnnotationKind::Imperium(command) if symbol_name(&result, command.name) == "emit"
    ));
    assert!(
        matches!(program.stmts[1].annotations[1].kind, AnnotationKind::Statement(_)),
        "@ alias remains generic in phase 01"
    );
}

#[test]
fn keeps_other_cli_metadata_annotations_generic() {
    let result = parse_ok(
        r#"
@ descriptio "Compile source files"
@ imperia "jobs" ex jobsModulum
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    assert_eq!(program.stmts[0].annotations.len(), 2);

    let AnnotationKind::Statement(description) = &program.stmts[0].annotations[0].kind else {
        panic!("@ descriptio should remain generic in phase 01");
    };
    assert_eq!(symbol_name(&result, description.name.name), "descriptio");
    assert_eq!(description.args.len(), 1);

    let AnnotationKind::Statement(imperia) = &program.stmts[0].annotations[1].kind else {
        panic!("@ imperia should remain generic in phase 01");
    };
    assert_eq!(symbol_name(&result, imperia.name.name), "imperia");
    assert_eq!(imperia.args.len(), 3);
}

#[test]
fn parses_canonical_optio_annotation_as_structured_ast() {
    let result = parse_ok(
        r#"
@ optio target brevis "t" longum "target" typus lista<textus> descriptio "Target language" ubique vel "rust"
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    let AnnotationKind::Optio(optio) = &program.stmts[0].annotations[0].kind else {
        panic!("expected structured @ optio");
    };

    assert_eq!(symbol_name(&result, optio.binding.name), "target");
    assert_eq!(symbol_name(&result, optio.short.expect("short")), "t");
    assert_eq!(symbol_name(&result, optio.long.expect("long")), "target");
    assert_eq!(symbol_name(&result, optio.description.expect("description")), "Target language");
    assert!(!optio.flag);
    assert!(optio.global);
    assert!(optio.default.is_some());

    let ty = optio.ty.as_ref().expect("explicit type");
    let TypeExprKind::Named(name, params) = &ty.kind else {
        panic!("expected named option type");
    };
    assert_eq!(symbol_name(&result, name.name), "lista");
    assert_eq!(params.len(), 1);
    let TypeExprKind::Named(param_name, param_params) = &params[0].kind else {
        panic!("expected named type parameter");
    };
    assert_eq!(symbol_name(&result, param_name.name), "textus");
    assert!(param_params.is_empty());
}

#[test]
fn parses_bivalens_optio_type_as_boolean_flag() {
    let result = parse_ok(
        r#"
@ optio verbose longum "verbose" typus bivalens
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    let AnnotationKind::Optio(optio) = &program.stmts[0].annotations[0].kind else {
        panic!("expected structured @ optio");
    };

    assert_eq!(symbol_name(&result, optio.binding.name), "verbose");
    assert!(optio.flag, "typus bivalens marks a boolean flag");
    assert!(
        optio.default.is_none(),
        "absent boolean flag defaults are applied by CLI validation/lowering"
    );
}

#[test]
fn parses_non_string_cli_default_expressions() {
    let result = parse_ok(
        r#"
@ optio limit longum "limit" typus numerus vel 100
@ optio strict longum "strict" typus bivalens vel verum
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");

    let AnnotationKind::Optio(limit) = &program.stmts[0].annotations[0].kind else {
        panic!("expected structured numeric @ optio");
    };
    let Some(default) = &limit.default else {
        panic!("expected numeric default");
    };
    assert!(matches!(default.kind, ExprKind::Literal(Literal::Integer(100))));

    let AnnotationKind::Optio(strict) = &program.stmts[0].annotations[1].kind else {
        panic!("expected structured boolean @ optio");
    };
    let Some(default) = &strict.default else {
        panic!("expected boolean default");
    };
    assert!(strict.flag);
    assert!(matches!(default.kind, ExprKind::Literal(Literal::Bool(true))));
}

#[test]
fn omitted_optio_type_is_left_for_textus_defaulting() {
    let result = parse_ok(
        r#"
@ optio output longum "output" descriptio "Output path"
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    let AnnotationKind::Optio(optio) = &program.stmts[0].annotations[0].kind else {
        panic!("expected structured @ optio");
    };

    assert_eq!(symbol_name(&result, optio.binding.name), "output");
    assert!(
        optio.ty.is_none(),
        "omitted typus means textus in later CLI validation/lowering"
    );
}

#[test]
fn parses_operandus_annotation_with_rest_type_global_and_default() {
    let result = parse_ok(
        r#"
@ operandus ceteri lista<textus> files descriptio "Input files" ubique vel "src"
incipit argumenta args {}
"#,
    );

    let program = result.program.as_ref().expect("program");
    let AnnotationKind::Operandus(operandus) = &program.stmts[0].annotations[0].kind else {
        panic!("expected structured @ operandus");
    };

    assert!(operandus.rest);
    assert_eq!(symbol_name(&result, operandus.binding.name), "files");
    assert_eq!(symbol_name(&result, operandus.description.expect("description")), "Input files");
    assert!(operandus.global);
    assert!(operandus.default.is_some());

    let TypeExprKind::Named(name, params) = &operandus.ty.kind else {
        panic!("expected named operand type");
    };
    assert_eq!(symbol_name(&result, name.name), "lista");
    assert_eq!(params.len(), 1);
}

#[test]
fn cli_option_parser_rejects_historical_type_first_and_bare_bivalens_forms() {
    let type_first = parse_program(
        r#"
@ optio textus output longum "output"
incipit argumenta args {}
"#,
    );
    assert!(
        type_first
            .errors
            .iter()
            .any(|err| err.message.contains("invalid @ optio modifier")),
        "type-first @ optio must not parse as canonical syntax"
    );

    let bare_bivalens = parse_program(
        r#"
@ optio verbose longum "verbose" bivalens
incipit argumenta args {}
"#,
    );
    assert!(
        bare_bivalens
            .errors
            .iter()
            .any(|err| err.message.contains("invalid @ optio modifier")),
        "bare bivalens modifier must not parse as canonical syntax"
    );
}

#[test]
fn malformed_cli_and_imperium_annotations_report_parse_errors() {
    assert_parse_error_contains(
        r#"
@ cli
incipit argumenta args {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ cli "faber" extra
incipit argumenta args {}
"#,
        "unexpected token after @ cli name",
    );

    assert_parse_error_contains(
        r#"
@ imperium
functio run() {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ imperium "run" extra
functio run() {}
"#,
        "unexpected token after @ imperium name",
    );
}

#[test]
fn malformed_optio_annotations_report_parse_errors() {
    assert_parse_error_contains(
        r#"
@ optio
incipit argumenta args {}
"#,
        "expected identifier",
    );

    assert_parse_error_contains(
        r#"
@ optio output brevis
incipit argumenta args {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ optio output longum
incipit argumenta args {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ optio output descriptio
incipit argumenta args {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ optio output typus
incipit argumenta args {}
"#,
        "expected identifier",
    );

    assert_parse_error_contains(
        r#"
@ optio output vel
incipit argumenta args {}
"#,
        "expected expression",
    );
}

#[test]
fn malformed_operandus_annotations_report_parse_errors() {
    assert_parse_error_contains(
        r#"
@ operandus textus
incipit argumenta args {}
"#,
        "expected identifier",
    );

    assert_parse_error_contains(
        r#"
@ operandus textus input descriptio
incipit argumenta args {}
"#,
        "expected string",
    );

    assert_parse_error_contains(
        r#"
@ operandus textus input longum "input"
incipit argumenta args {}
"#,
        "invalid @ operandus modifier",
    );

    assert_parse_error_contains(
        r#"
@ operandus textus input vel
incipit argumenta args {}
"#,
        "expected expression",
    );
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

    let program = result
        .program
        .expect("parser should still produce a program");
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

    let program = result
        .program
        .expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1);

    let StmtKind::Incipit(entry) = &program.stmts[0].kind else {
        panic!("expected incipit statement");
    };

    let crate::syntax::IfBody::Block(block) = &entry.body else {
        panic!("expected block body");
    };

    assert_eq!(
        block.stmts.len(),
        1,
        "recovery should preserve valid statements inside the block"
    );
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

    let program = result
        .program
        .expect("parser should still produce a program");
    assert_eq!(program.stmts.len(), 1, "recovery should preserve the following declaration");

    let StmtKind::Func(func) = &program.stmts[0].kind else {
        panic!("expected function declaration");
    };

    assert_eq!(
        program.stmts[0].annotations.len(),
        1,
        "annotation should remain attached after recovery"
    );
    assert!(func.body.is_some());
}

#[test]
fn parses_following_variable_after_if_body_error() {
    let result = parse_program(
        r#"
si verum x
fixum numerus postea ← 2
"#,
    );

    assert_eq!(result.errors.len(), 1, "expected one parse error");

    let program = result
        .program
        .expect("parser should still produce a program");
    assert_eq!(
        program.stmts.len(),
        1,
        "recovery should preserve the following variable declaration"
    );

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
fixum numerus count ← 1
varia [head, ceteri tail] ← values
@ futura
functio mitte(prae typus T, de textus nomen sponte ut alias vel "anon", ex numerus item, ceteri T reliqua) curata allocator errata Error exitus 7 immutata iacit optiones Opts → textus {}
typus Nomen = textus
ordo Status { Bonus = 1, Malus = -1, Textus = "x" }
discretio Resultatus<T> { Ok { T valor }, Err { textus nuntius } }
"#,
    );

    assert_eq!(result.program.as_ref().expect("program").directives.len(), 1);

    let program = result
        .program
        .as_ref()
        .expect("parser should produce a program");
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
    assert!(func.params[0].sponte);
    assert!(matches!(func.params[0].mode, ParamMode::Ref));
    assert_eq!(
        symbol_name(&result, func.params[0].alias.as_ref().expect("alias").name),
        "alias"
    );
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
fn parses_infer_marker_as_type_annotation() {
    let result = parse_ok(
        r#"
fixum _ name ← "Marcus"
varia _ count ← 1
functio answer() → _ { redde 42 }
"#,
    );

    let program = result.program.as_ref().expect("program");

    let StmtKind::Var(name_var) = &program.stmts[0].kind else {
        panic!("expected variable declaration");
    };
    assert!(matches!(
        name_var.ty.as_ref().expect("explicit infer marker").kind,
        TypeExprKind::Infer
    ));

    let StmtKind::Func(func) = &program.stmts[2].kind else {
        panic!("expected function declaration");
    };
    assert!(matches!(
        func.ret.as_ref().expect("explicit inferred return").kind,
        TypeExprKind::Infer
    ));
}

#[test]
fn bare_inferred_variable_declaration_requires_marker() {
    assert_parse_error_contains(r#"fixum name ← "Marcus""#, "expected");
    assert_parse_error_contains(r#"varia count ← 1"#, "expected");
}

#[test]
fn cura_rejects_unknown_allocator_kind() {
    assert_parse_error_contains(r#"cura "general" fixum _ alloc {}"#, "expected allocator kind");
}

#[test]
fn parses_class_interface_and_test_keywords() {
    let result = parse_ok(
        r#"
abstractus genus Animal<T> sub Vivens implet Canens, Currens {
    @ futura
    functio canta(textus vox) → vacuum {}
    generis textus nomen: "leo"
    nexum numerus aetas: 3
}

pactum Canens<T> {
    @ externa
    functio canta(textus vox) iacit → vacuum
    @ externa
    functio aliter() → textus
}
probandum "suite" tag "parser" {
    praepara omnia {}
    postpara {}
    proba "case" omitte "skip" futurum "later" solum tag "focus" temporis 5 metior repete 2 fragilis 1 requirit "net" solum_in "ci" {}
    probandum "nested" { proba "inner" {} }
}
"#,
    );

    let program = result
        .program
        .as_ref()
        .expect("parser should produce a program");
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
    assert_eq!(interface.methods.len(), 2);
    assert_eq!(interface.methods[0].modifiers.len(), 1);

    let StmtKind::Probandum(test) = &program.stmts[2].kind else {
        panic!("expected probandum declaration");
    };
    assert_eq!(test.modifiers.len(), 1);
    assert!(matches!(test.modifiers[0], ProbaModifier::Tag(_)));
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
    assert!(matches!(
        case.modifiers[8],
        ProbaModifier::Requirit(_) | ProbaModifier::SolumIn(_)
    ));
    assert!(
        case.modifiers
            .iter()
            .any(|modifier| matches!(modifier, ProbaModifier::Requirit(_))),
        "expected requirit modifier"
    );
    assert!(
        case.modifiers
            .iter()
            .any(|modifier| matches!(modifier, ProbaModifier::SolumIn(_))),
        "expected solumIn modifier"
    );
}

#[test]
fn inline_union_types_parse_flat() {
    let result = parse_ok("typus Maybe = textus ∪ numerus ∪ nihil");
    let program = result.program.as_ref().expect("program");
    let StmtKind::TypeAlias(alias) = &program.stmts[0].kind else {
        panic!("expected type alias");
    };
    let TypeExprKind::Union(members) = &alias.ty.kind else {
        panic!("expected flat union");
    };
    assert_eq!(members.len(), 3);
}

#[test]
fn numeric_test_modifiers_require_numbers() {
    assert_parse_error_contains(
        r#"probandum "suite" { proba "case" temporis { adfirma verum } }"#,
        "expected integer after 'temporis'",
    );
    assert_parse_error_contains(
        r#"probandum "suite" { proba "case" repete { adfirma verum } }"#,
        "expected integer after 'repete'",
    );
    assert_parse_error_contains(
        r#"probandum "suite" { proba "case" fragilis { adfirma verum } }"#,
        "expected integer after 'fragilis'",
    );
}

#[test]
fn proba_modifiers_must_follow_name() {
    assert_parse_error_contains(r#"proba solum tag "parser" "parses input" {}"#, "expected string");
}

#[test]
fn probandum_modifiers_must_follow_name() {
    assert_parse_error_contains(r#"probandum solum tag "parser" "suite" {}"#, "expected string");
}

#[test]
fn parses_control_flow_transfer_and_clause_keywords() {
    let result = parse_ok(
        r#"
si verum ergo nota "yes" cape err {} sin falsum ergo redde 1 secus ergo tacet
dum verum { perge } cape err {}
itera pro items fixum item ergo redde item cape err {}
elige value { casu 1 ergo redde 1 ceterum ergo tacet } cape err {}
discerne omnia value, other { casu Ok ut result, _ ergo redde result ceterum ergo mori "bad" }
custodi { si verum ergo redde 1 si falsum ergo nota "no" }
fac {} cape err {} dum verum
adfirma verum, "msg"
nota "a", "b"
vide "c"
mone "d"
scribe "e"
redde 1
rumpe
perge
iace "err"
mori "panic"
tacet
"#,
    );

    let program = result
        .program
        .as_ref()
        .expect("parser should produce a program");
    assert_eq!(program.stmts.len(), 18);

    let StmtKind::Si(if_stmt) = &program.stmts[0].kind else {
        panic!("expected if statement");
    };
    assert!(matches!(if_stmt.then, IfBody::Ergo(_)));
    assert!(if_stmt.catch.is_some());
    let SecusClause::Sin(sin_stmt) = if_stmt.else_.as_ref().expect("else clause") else {
        panic!("expected sin clause");
    };
    assert!(matches!(sin_stmt.then, IfBody::Ergo(_)));
    assert!(matches!(
        sin_stmt.else_.as_ref().expect("secus clause"),
        SecusClause::Stmt { .. }
    ));

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
    assert!(matches!(iter_stmt.body, IfBody::Ergo(_)));

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

    assert!(matches!(program.stmts[7].kind, StmtKind::Adfirma(_)));
    match &program.stmts[8].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Nota)),
        _ => panic!("expected diagnostic statement"),
    }
    match &program.stmts[9].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Vide)),
        _ => panic!("expected vide statement"),
    }
    match &program.stmts[10].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Mone)),
        _ => panic!("expected mone statement"),
    }
    match &program.stmts[11].kind {
        StmtKind::Scribe(stmt) => assert!(matches!(stmt.kind, ScribeKind::Scribe)),
        _ => panic!("expected scribe compatibility statement"),
    }
    assert!(matches!(program.stmts[12].kind, StmtKind::Redde(_)));
    assert!(matches!(program.stmts[13].kind, StmtKind::Rumpe(_)));
    assert!(matches!(program.stmts[14].kind, StmtKind::Perge(_)));
    assert!(matches!(program.stmts[15].kind, StmtKind::Iace(_)));
    assert!(matches!(program.stmts[16].kind, StmtKind::Mori(_)));
    assert!(matches!(program.stmts[17].kind, StmtKind::Tacet(_)));
}

#[test]
fn parses_ergo_symbol_as_statement_tail() {
    let result = parse_ok(
        r#"
si verum ∴ nota "yes"
dum falsum ∴ tacet
itera pro items fixum item ∴ redde item
"#,
    );
    let program = result.program.as_ref().expect("program");
    assert_eq!(program.stmts.len(), 3);
    let StmtKind::Si(stmt) = &program.stmts[0].kind else {
        panic!("expected si statement");
    };
    assert!(matches!(stmt.then, IfBody::Ergo(_)));
    let StmtKind::Dum(stmt) = &program.stmts[1].kind else {
        panic!("expected dum statement");
    };
    assert!(matches!(stmt.body, IfBody::Ergo(_)));
}

#[test]
fn parses_compact_closure_forms() {
    let result = parse_ok(
        r#"
fixum _ a ← users.filtrata(_ user ∴ non user.activus)
fixum _ b ← users.filtrata(User user ∴ non user.activus)
fixum _ c ← numeri.compone((_ a, _ b) ∴ a + b)
fixum _ d ← numeri.compone((numerus a, numerus b) ∴ fac { redde a + b })
fixum _ e ← texts.mappata(textus s → numerus ⇥ ParseError ∴ fac { redde parse(s) } cape err { redde 0 })
"#,
    );
    let program = result.program.as_ref().expect("program");
    assert_eq!(program.stmts.len(), 5);

    for stmt in &program.stmts[..3] {
        let StmtKind::Var(var) = &stmt.kind else {
            panic!("expected variable");
        };
        let ExprKind::Call(call) = &var.init.as_ref().expect("init").kind else {
            panic!("expected call");
        };
        let ExprKind::Clausura(closure) = &call.args[0].value.kind else {
            panic!("expected compact closure");
        };
        assert!(matches!(closure.body, ClausuraBody::Expr(_)));
    }

    let StmtKind::Var(block_var) = &program.stmts[3].kind else {
        panic!("expected variable");
    };
    let ExprKind::Call(call) = &block_var.init.as_ref().expect("init").kind else {
        panic!("expected call");
    };
    let ExprKind::Clausura(closure) = &call.args[0].value.kind else {
        panic!("expected compact closure");
    };
    assert_eq!(closure.params.len(), 2);
    assert!(matches!(closure.body, ClausuraBody::Fac(_)));

    let StmtKind::Var(fallible_var) = &program.stmts[4].kind else {
        panic!("expected variable");
    };
    let ExprKind::Call(call) = &fallible_var.init.as_ref().expect("init").kind else {
        panic!("expected call");
    };
    let ExprKind::Clausura(closure) = &call.args[0].value.kind else {
        panic!("expected compact closure");
    };
    assert!(closure.ret.is_some());
    assert!(closure.err.is_some());
    assert!(matches!(closure.body, ClausuraBody::Fac(_)));
}

#[test]
fn compact_closure_rejects_bare_block_and_fac_dum_body() {
    assert_parse_error_contains(
        "fixum _ inactive ← users.filtrata(_ user ∴ { redde non user.activus })",
        "closure block body must use 'fac'",
    );
    assert_parse_error_contains(
        "fixum _ inactive ← users.filtrata(_ user ∴ fac { redde non user.activus } dum user.activus)",
        "closure fac body cannot use 'dum'",
    );
}

#[test]
fn parses_structured_cape_attachment_targets() {
    let result = parse_ok(
        r#"
si verum { tacet } cape err {} sin falsum { tacet } cape err {} secus { tacet } cape err {}
dum verum { tacet } cape err {}
fac { tacet } cape err {}
"#,
    );
    let program = result.program.as_ref().expect("program");
    assert_eq!(program.stmts.len(), 3);

    let StmtKind::Si(si_stmt) = &program.stmts[0].kind else {
        panic!("expected si");
    };
    assert!(si_stmt.catch.is_some());
    let SecusClause::Sin(sin_stmt) = si_stmt.else_.as_ref().expect("sin") else {
        panic!("expected sin");
    };
    assert!(sin_stmt.catch.is_some());
    let SecusClause::Block { catch, .. } = sin_stmt.else_.as_ref().expect("secus") else {
        panic!("expected secus block");
    };
    assert!(catch.is_some());

    let StmtKind::Dum(dum_stmt) = &program.stmts[1].kind else {
        panic!("expected dum");
    };
    assert!(dum_stmt.catch.is_some());

    let StmtKind::Fac(fac_stmt) = &program.stmts[2].kind else {
        panic!("expected fac");
    };
    assert!(fac_stmt.catch.is_some());
}

#[test]
fn rejects_bare_block_cape_and_legacy_tempta() {
    assert_parse_error_contains(
        r#"functio invalid() → vacuum { { tacet } cape err { tacet } }"#,
        "expected expression",
    );
    assert_parse_error_contains(
        r#"functio old() → vacuum { tempta { tacet } cape err { tacet } }"#,
        "tempta is no longer canonical",
    );
}

#[test]
fn rejects_non_ergo_tacet_branch_forms() {
    assert_parse_error_contains("si verum tacet", "expected block or 'ergo'");
    assert_parse_error_contains("si verum ergo tacet secus tacet", "expected block or 'ergo'");
    assert_parse_error_contains("elige value { casu 1 tacet }", "expected block or 'ergo'");
    assert_parse_error_contains("dum verum tacet", "expected block or 'ergo'");
    assert_parse_error_contains("incipit tacet", "expected '{'");
}

#[test]
fn parses_entry_resource_endpoint_and_extract_keywords() {
    let result = parse_ok(
        r#"
incipit argumenta args exitus 1 {}
incipiet argumenta argv {}
cura "arena" fixum _ alloc {}
cura "page" fixum _ page {}
ad "/salve" (request, sparge extra) → textus pro res ut alias {} cape err {}
ex source varia nomen ut name, ceteri reliqua
"#,
    );

    let program = result
        .program
        .as_ref()
        .expect("parser should produce a program");
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
    assert!(matches!(async_main.body, IfBody::Block(_)));

    let StmtKind::Cura(arena_stmt) = &program.stmts[2].kind else {
        panic!("expected arena cura");
    };
    assert!(matches!(arena_stmt.kind, CuraKind::Arena));

    let StmtKind::Cura(page_stmt) = &program.stmts[3].kind else {
        panic!("expected bound page cura");
    };
    assert!(matches!(page_stmt.kind, CuraKind::Page));
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
    assert_eq!(
        symbol_name(&result, extract.fields[0].alias.as_ref().expect("field alias").name),
        "name"
    );
    assert_eq!(
        symbol_name(&result, extract.rest.as_ref().expect("rest binding").name),
        "reliqua"
    );
}

#[test]
fn parses_curata_allocator_alias() {
    let result = parse_ok(r#"functio greet(textus name) curata alloc ut a → textus { redde name }"#);
    let program = result.program.as_ref().expect("program");
    let StmtKind::Func(func) = &program.stmts[0].kind else {
        panic!("expected function declaration");
    };
    let crate::syntax::FuncModifier::Curata { required, alias: Some(alias) } = &func.modifiers[0] else {
        panic!("expected curata alias modifier");
    };
    assert_eq!(symbol_name(&result, required.name), "alloc");
    assert_eq!(symbol_name(&result, alias.name), "a");
}

#[test]
fn parses_function_alternate_exit_type() {
    let result = parse_ok(r#"functio divide(numerus a, numerus b) → numerus ⇥ textus { redde a }"#);
    let program = result.program.as_ref().expect("program");
    let StmtKind::Func(func) = &program.stmts[0].kind else {
        panic!("expected function declaration");
    };
    assert!(matches!(func.ret.as_ref().map(|ty| &ty.kind), Some(TypeExprKind::Named(_, _))));
    assert!(matches!(func.err.as_ref().map(|ty| &ty.kind), Some(TypeExprKind::Named(_, _))));
    let Some(err) = &func.err else {
        panic!("expected alternate-exit type");
    };
    let TypeExprKind::Named(name, params) = &err.kind else {
        panic!("expected named alternate-exit type");
    };
    assert!(params.is_empty());
    assert_eq!(symbol_name(&result, name.name), "textus");
}

#[test]
fn parses_pactum_method_alternate_exit_type() {
    let result = parse_ok(
        r#"
pactum Divisor {
  functio divide(numerus a, numerus b) → numerus ⇥ textus
}
"#,
    );
    let program = result.program.as_ref().expect("program");
    let StmtKind::Interface(interface) = &program.stmts[0].kind else {
        panic!("expected pactum declaration");
    };
    assert_eq!(interface.methods.len(), 1);
    assert!(interface.methods[0].ret.is_some());
    assert!(interface.methods[0].err.is_some());
}

#[test]
fn parses_failable_function_type() {
    let result = parse_ok(r#"typus Op = (numerus) → numerus ⇥ textus"#);
    let program = result.program.as_ref().expect("program");
    let StmtKind::TypeAlias(alias) = &program.stmts[0].kind else {
        panic!("expected type alias");
    };
    let TypeExprKind::Func(func) = &alias.ty.kind else {
        panic!("expected function type");
    };
    assert_eq!(func.params.len(), 1);
    assert!(func.err.is_some());
}

#[test]
fn legacy_si_declaration_and_type_forms_are_rejected() {
    // Legacy declaration optionality (si as prefix in params, with/without ownership modes)
    assert_parse_error_contains(r#"functio f(si textus name) → vacuum {}"#, "expected identifier");
    assert_parse_error_contains(r#"functio f(de si textus handle) → vacuum {}"#, "expected identifier");
    assert_parse_error_contains(r#"functio f(si de textus handle) → vacuum {}"#, "expected identifier");

    // Legacy si in genus field position
    assert_parse_error_contains(r#"genus User { si textus email }"#, "expected identifier");

    // Legacy nullable type syntax in returns, typus aliases, and local var decls
    assert_parse_error_contains(r#"functio find() → si textus { redde nihil }"#, "expected identifier");
    assert_parse_error_contains(r#"typus MaybeText = si textus"#, "expected identifier");
    assert_parse_error_contains(r#"fixum si textus maybe ← nihil"#, "expected identifier");
    assert_parse_error_contains(r#"varia si numerus maybe ← nihil"#, "expected identifier");
}

#[test]
fn legacy_suffix_nullable_and_reversed_markers_are_rejected() {
    // Legacy T? suffix form ( ? is only for optional chaining, not type nullability)
    assert_parse_error_contains(r#"fixum textus? maybe ← nihil"#, "expected identifier");
    assert_parse_error_contains(r#"functio find() → textus? { redde nihil }"#, "expected expression");
    assert_parse_error_contains(r#"typus MaybeText = textus?"#, "expected expression");

    // Reversed declaration marker order (fixus before sponte) must be rejected
    // (canonical order is <type> <name> [sponte] [fixus] ...)
    assert_parse_error_contains(
        r#"functio f(textus name fixus sponte) → vacuum {}"#,
        "unexpected 'sponte' after 'fixus'",
    );
    assert_parse_error_contains(
        r#"genus User { textus email fixus sponte }"#,
        "unexpected 'sponte' after 'fixus'",
    );
}
