use crate::codegen::rust::TestSelection;
use crate::codegen::{self, Target};
use crate::hir::{
    DefId, HirArrayElement, HirBlock, HirCallArg, HirCasuArm, HirEnum, HirExpr, HirExprKind, HirField, HirFunction,
    HirId, HirImport, HirImportItem, HirInterface, HirItem, HirItemKind, HirIteraMode, HirLiteral, HirParam,
    HirParamMode, HirPattern, HirProgram, HirScribeKind, HirStmt, HirStmtKind, HirStruct, HirTestMetadata,
    HirTestModifier, HirTypeAlias, HirVariant, HirVariantField,
};
use crate::lexer::{Interner, Span};
use crate::semantic::{FuncSig, InferVar, Mutability, ParamMode, ParamType, Primitive, Type, TypeTable};
use crate::syntax::Visibility;

#[path = "tests/ad_test.rs"]
mod ad;
#[path = "tests/calls_test.rs"]
mod calls;
#[path = "tests/collections_test.rs"]
mod collections;
#[path = "tests/decl_test.rs"]
mod decl;
#[path = "tests/dynamic_test.rs"]
mod dynamic;
#[path = "tests/failable_test.rs"]
mod failable;
#[path = "tests/optional_test.rs"]
mod optional;
#[path = "tests/types_test.rs"]
mod types_test;

fn span() -> Span {
    Span::default()
}

fn test_body(vacuum: crate::semantic::TypeId, bivalens: crate::semantic::TypeId) -> HirBlock {
    HirBlock {
        stmts: vec![HirStmt {
            id: crate::hir::HirId(2),
            kind: HirStmtKind::Expr(HirExpr {
                id: crate::hir::HirId(3),
                kind: HirExprKind::Adfirma(
                    Box::new(HirExpr {
                        id: crate::hir::HirId(4),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    None,
                ),
                ty: Some(vacuum),
                span: span(),
            }),
            span: span(),
        }],
        expr: None,
        span: span(),
    }
}

fn test_item(
    def_id: u32,
    name: crate::lexer::Symbol,
    vacuum: crate::semantic::TypeId,
    bivalens: crate::semantic::TypeId,
    suite_path: Vec<crate::lexer::Symbol>,
    modifiers: Vec<HirTestModifier>,
) -> HirItem {
    HirItem {
        id: crate::hir::HirId(def_id + 1),
        def_id: DefId(def_id),
        kind: HirItemKind::Function(HirFunction {
            cli_args: None,
            name,
            type_params: Vec::new(),
            params: Vec::new(),
            ret_ty: Some(vacuum),
            err_ty: None,
            body: Some(test_body(vacuum, bivalens)),
            is_async: false,
            is_generator: false,
            test: Some(HirTestMetadata { name, suite_path, modifiers, span: span() }),
        }),
        span: span(),
    }
}

#[test]
fn emits_rust_function_and_entry_via_codegen_dispatch() {
    let mut interner = Interner::new();
    let name_f = interner.intern("f");
    let name_x = interner.intern("x");
    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(0),
            def_id: DefId(1),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: name_f,
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(2),
                    name: name_x,
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    optional: false,
                    sponte: false,
                    fixus: false,
                    default: None,
                    span: span(),
                }],
                ret_ty: Some(numerus),
                err_ty: None,
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(3),
                        kind: HirStmtKind::Redde(Some(HirExpr {
                            id: HirId(4),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        })),
                        span: span(),
                    }],
                    expr: None,
                    span: span(),
                }),
                is_async: false,
                is_generator: false,
                test: None,
            }),
            span: span(),
        }],
        entry: Some(HirBlock { stmts: Vec::new(), expr: None, span: span() }),
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("fn f(x: i64) -> i64"));
    assert!(rust.code.contains("fn main() {"));
}

#[test]
fn emits_text_concatenation_without_invalid_string_add() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio greet(textus name) → textus {
    redde "Hello, " + name
}

incipit {
    varia textus s ← "hello"
    s ⊕ " world"
    varia numerus n ← 1
    n ⊕ 2
    nota greet(s)
}
"#;

    let result = compiler.compile_str("string-concat.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("format!(\"{}{}\", \"Hello, \".to_string(), name)"));
    assert!(rust.code.contains("s.push_str(&\" world\".to_string())"));
    assert!(rust.code.contains("n += 2"));
    assert!(!rust.code.contains("+ name"));
    assert!(!rust.code.contains("+= \" world\".to_string()"));
}

#[test]
fn emits_iterable_rust_ranges_for_itera_ab() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    itera ab 0‥3 fixum i {
        nota i
    }
    itera ab 0…3 fixum j {
        nota j
    }
    itera ab 0‥6 per 2 fixum k {
        nota k
    }
    itera ab 6‥0 per -2 fixum n {
        nota n
    }
}
"#;

    let result = compiler.compile_str("range-iteration.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("for i in { let __faber_start: i64 = 0"));
    assert!(rust.code.contains("let __faber_limit: i64 = __faber_end"));
    assert!(rust
        .code
        .contains("let __faber_limit: i64 = __faber_end + __faber_step.signum()"));
    assert!(rust.code.contains("let __faber_step: i64 = 2"));
    assert!(rust.code.contains("let __faber_step: i64 = -2"));
    assert!(!rust.code.contains("for i in (0, 3)"));
}

#[test]
fn emits_integral_fractus_literals_as_rust_floats() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio average(fractus a, fractus b) → fractus {
    redde (a + b) / 2.0
}

incipit {
    nota average(3.0, 7.0)
}
"#;

    let result = compiler.compile_str("float-literals.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("return (a + b) / 2.0;"));
    assert!(rust.code.contains("average(3.0, 7.0)"));
    assert!(!rust.code.contains("average(3, 7)"));
}

#[test]
fn emits_unresolved_ad_dispatch_for_rust() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "fasciculus:lege" ("hello.txt") → textus content ⇥ textus {
        nota content
    }
}
"#;

    let result = compiler.compile_str("ad-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("fn __faber_ad<T, A>(capability: &str, _args: A) -> Result<T, String>"));
    assert!(rust
        .code
        .contains("match __faber_ad::<String, _>(\"fasciculus:lege\", (\"hello.txt\".to_string(),))"));
    assert!(rust.code.contains("E_NO_ROUTE: unresolved capability"));
    assert!(rust.code.contains("let content = __faber_result;"));
}

#[test]
fn unresolved_ad_can_route_to_cape_handler_in_rust() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "fasciculus:lege" ("hello.txt") → textus content ⇥ textus {
        nota content
    } cape err {
        nota err
    }
}
"#;

    let result = compiler.compile_str("ad-cape-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("Err(__faber_err) => {"));
    assert!(rust.code.contains("let err = __faber_err;"));
    assert!(rust.code.contains("println!(\"{}\", err);"));
}

#[test]
fn uncaught_ad_marks_rust_function_failable() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio legit() → textus {
    ad "fasciculus:lege" ("hello.txt") → textus content ⇥ textus {
        redde content
    }
    redde "unreachable"
}
"#;

    let result = compiler.compile_str("ad-failable-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("fn legit() -> Result<String, String>"));
    assert!(rust.code.contains("return Err(__faber_err);"));
}

#[test]
fn ad_rejects_legacy_pro_success_binding() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "fasciculus:lege" ("hello.txt") → pro content {
        nota content
    }
}
"#;

    let result = compiler.compile_str("ad-legacy-pro.fab", source);

    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("ad success bindings use '→ Type name', not '→ pro name'")
    }));
}

#[test]
fn ad_cape_requires_explicit_error_channel_type() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "fasciculus:lege" ("hello.txt") → textus content {
        nota content
    } cape err {
        nota err
    }
}
"#;

    let result = compiler.compile_str("ad-cape-missing-error-type.fab", source);

    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ad cape handlers require an explicit ⇥ error type")));
}

#[test]
fn emits_fractus_arithmetic_casts_for_numerus_operands() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Circle {
    numerus radius
}

incipit {
    fixum _ radius ← 5
    fixum _ circle ← Circle { radius = radius }
    fixum _ tau ← 2 * 3.14159
    fixum _ area ← 3.14159 * circle.radius
    nota tau, area
}
"#;

    let result = compiler.compile_str("mixed-fractus-arithmetic.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("(2 as f64) * 3.14159"));
    assert!(rust.code.contains("3.14159 * (circle.radius as f64)"));
}

#[test]
fn emits_contextual_fractus_integer_division_as_float_division() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio divide(numerus a, numerus b) → fractus {
    redde a / b
}

incipit {
    nota divide(3, 2)
}
"#;

    let result = compiler.compile_str("contextual-fractus-division.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("return (a as f64) / (b as f64);"));
}

#[test]
fn emits_qualified_enum_variants_for_values_constructors_and_patterns() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
discretio Event {
    Click { numerus x, numerus y },
    Quit
}

functio handle(Event e) → vacuum {
    discerne e {
        casu Click fixum x, y {
            nota x, y
        }
        casu Quit {
            nota "quit"
        }
    }
}

incipit {
    fixum Event e1 ← finge Click { y = 2, x = 1 } ∷ Event
    fixum Event e2 ← finge Quit ∷ Event
    handle(e1)
    handle(e2)
}
"#;

    let result = compiler.compile_str("qualified-enum-variants.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("Event::Click { x, y } =>"));
    assert!(rust.code.contains("Event::Quit =>"));
    assert!(rust
        .code
        .contains("let e1: Event = Event::Click { x: 1, y: 2 };"));
    assert!(rust.code.contains("let e2: Event = Event::Quit;"));
}

#[test]
fn rejects_unknown_finge_variant_fields_before_codegen() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
discretio Event {
    Click { numerus x, numerus y }
}

incipit {
    fixum Event e1 ← finge Click { y = 2, z = 1 } ∷ Event
}
"#;

    let result = compiler.compile_str("unknown-finge-field.fab", source);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown variant field")));
}

#[test]
fn emits_async_futura_functions_and_entry_block_on() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
@ futura
functio fetchData() → textus {
    redde "data loaded"
}

incipiet {
    fixum _ data ← cede fetchData()
    nota data
}
"#;

    let result = compiler.compile_str("incipiet-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("async fn fetchData() -> String"));
    assert!(rust
        .code
        .contains("fn __faber_block_on<F: std::future::Future>"));
    assert!(rust.code.contains("__faber_block_on(async {"));
    assert!(rust.code.contains("fetchData().await"));
}

#[test]
fn emits_cursor_functions_as_vec_producers() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
@ cursor
functio rangeSync(numerus n) → numerus {
    itera ab 0‥n fixum i {
        cede i
    }
}

@ futura
@ cursor
functio rangeAsync(numerus n) → numerus {
    itera ab 0‥n fixum i {
        cede i
    }
}

incipit {
    itera ex rangeSync(3) fixum num {
        nota num
    }
}
"#;

    let result = compiler.compile_str("cursor-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("fn rangeSync(n: i64) -> Vec<i64>"));
    assert!(rust
        .code
        .contains("async fn rangeAsync(n: i64) -> Vec<i64>"));
    assert!(rust
        .code
        .contains("let mut __faber_yielded: Vec<i64> = Vec::new();"));
    assert!(rust.code.contains("__faber_yielded.push(i);"));
    assert!(rust.code.contains("for __faber_item_"));
    assert!(rust.code.contains("in &(rangeSync(3))"));
    assert!(!rust.code.contains("i.await"));
}

#[test]
fn emits_radix_parse_for_hinted_numerus_conversio() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ hex ← "ff" ⇒ numerus<i32, Hex>
    fixum _ bin ← "1010" ⇒ numerus<i32, Bin>
    fixum _ oct ← "755" ⇒ numerus<i32, Oct>
    fixum _ dec ← "42" ⇒ numerus
    nota hex, bin, oct, dec
}
"#;

    let result = compiler.compile_str("radix-conversio.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("i64::from_str_radix(&(\"ff\".to_string()), 16).unwrap()"));
    assert!(rust
        .code
        .contains("i64::from_str_radix(&(\"1010\".to_string()), 2).unwrap()"));
    assert!(rust
        .code
        .contains("i64::from_str_radix(&(\"755\".to_string()), 8).unwrap()"));
    assert!(rust
        .code
        .contains("\"42\".to_string().parse::<i64>().unwrap()"));
}

#[test]
fn emits_wildcard_arm_for_elige_without_ceterum() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio greeting(textus language) → textus {
    elige language {
        casu "latin" { redde "Salve" }
        casu "english" { redde "Hello" }
    }
    redde "Hi"
}

incipit {
    fixum _ code ← 200
    elige code {
        casu 200 { nota "OK" }
        casu 404 { nota "Missing" }
    }
}
"#;

    let result = compiler.compile_str("elige-exhaustive.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("match language.as_str()"));
    assert!(rust.code.contains("_ => {},"));
    assert!(rust.code.contains("return \"Hi\".to_string();"));
}

#[test]
fn emits_metadata_driven_test_attributes() {
    let mut interner = Interner::new();
    let case_name = interner.intern("one plus one equals two");
    let blocked = interner.intern("blocked by maintenance");
    let suite = interner.intern("arithmetic suite");
    let types = TypeTable::new();
    let vacuum = types.primitive(Primitive::Vacuum);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(10),
            def_id: DefId(1_000_000),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: case_name,
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(vacuum),
                err_ty: None,
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(11),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: HirId(12),
                            kind: HirExprKind::Adfirma(
                                Box::new(HirExpr {
                                    id: HirId(13),
                                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                                    ty: Some(types.primitive(Primitive::Bivalens)),
                                    span: span(),
                                }),
                                None,
                            ),
                            ty: Some(vacuum),
                            span: span(),
                        }),
                        span: span(),
                    }],
                    expr: None,
                    span: span(),
                }),
                is_async: false,
                is_generator: false,
                test: Some(HirTestMetadata {
                    name: case_name,
                    suite_path: vec![suite],
                    modifiers: vec![HirTestModifier::Futurum(blocked)],
                    span: span(),
                }),
            }),
            span: span(),
        }],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("#[test]"));
    assert!(rust
        .code
        .contains("#[ignore = \"faber: futurum - blocked by maintenance\"]"));
    assert!(rust.code.contains("fn proba_1000000"));
}

#[test]
fn emits_solum_default_ignores_non_solum_tests() {
    let mut interner = Interner::new();
    let focused = interner.intern("focused case");
    let other = interner.intern("other case");
    let types = TypeTable::new();
    let vacuum = types.primitive(Primitive::Vacuum);
    let bivalens = types.primitive(Primitive::Bivalens);

    let program = HirProgram {
        items: vec![
            test_item(20, focused, vacuum, bivalens, Vec::new(), vec![HirTestModifier::Solum]),
            test_item(21, other, vacuum, bivalens, Vec::new(), Vec::new()),
        ],
        entry: None,
    };

    let output = crate::codegen::rust::generate_module_with_test_selection(&program, &types, &interner, None)
        .expect("rust codegen");
    let rust = output;

    assert!(rust
        .code
        .contains("#[ignore = \"faber: not selected by solum\"]"));
    assert!(rust.code.contains("fn proba_20"));
    assert!(rust.code.contains("fn proba_21"));
}

#[test]
fn emits_explicit_selector_ignores_for_name_suite_and_tag() {
    let mut interner = Interner::new();
    let selected = interner.intern("selected case");
    let wrong_name = interner.intern("wrong name");
    let outer = interner.intern("outer suite");
    let inner = interner.intern("inner suite");
    let smoke = interner.intern("smoke");
    let slow = interner.intern("slow");
    let types = TypeTable::new();
    let vacuum = types.primitive(Primitive::Vacuum);
    let bivalens = types.primitive(Primitive::Bivalens);

    let program = HirProgram {
        items: vec![
            test_item(
                30,
                selected,
                vacuum,
                bivalens,
                vec![outer, inner],
                vec![HirTestModifier::Tag(smoke)],
            ),
            test_item(
                31,
                wrong_name,
                vacuum,
                bivalens,
                vec![outer, inner],
                vec![HirTestModifier::Tag(smoke)],
            ),
            test_item(32, selected, vacuum, bivalens, vec![outer], vec![HirTestModifier::Tag(smoke)]),
            test_item(
                33,
                selected,
                vacuum,
                bivalens,
                vec![outer, inner],
                vec![HirTestModifier::Tag(slow)],
            ),
        ],
        entry: None,
    };

    let selection = TestSelection {
        name: Some("selected case".to_owned()),
        suite: Some("outer suite/inner suite".to_owned()),
        tag: Some("smoke".to_owned()),
    };
    let output =
        crate::codegen::rust::generate_module_with_test_selection(&program, &types, &interner, Some(selection))
            .expect("rust codegen");
    let rust = output;

    assert!(rust.code.contains("fn proba_30"));
    assert!(rust
        .code
        .contains("#[ignore = \"faber: not selected by name selected case\"]"));
    assert!(rust
        .code
        .contains("#[ignore = \"faber: not selected by suite outer suite/inner suite\"]"));
    assert!(rust
        .code
        .contains("#[ignore = \"faber: not selected by tag smoke\"]"));
}

#[test]
fn emits_source_ignore_reason_for_selected_test() {
    let mut interner = Interner::new();
    let name = interner.intern("selected ignored case");
    let reason = interner.intern("blocked by service");
    let types = TypeTable::new();
    let vacuum = types.primitive(Primitive::Vacuum);
    let bivalens = types.primitive(Primitive::Bivalens);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(30),
            def_id: DefId(30),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name,
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(vacuum),
                err_ty: None,
                body: Some(test_body(vacuum, bivalens)),
                is_async: false,
                is_generator: false,
                test: Some(HirTestMetadata {
                    name,
                    suite_path: Vec::new(),
                    modifiers: vec![HirTestModifier::Omitte(reason)],
                    span: span(),
                }),
            }),
            span: span(),
        }],
        entry: None,
    };

    let selection = TestSelection { name: Some("selected ignored case".to_owned()), suite: None, tag: None };
    let output =
        crate::codegen::rust::generate_module_with_test_selection(&program, &types, &interner, Some(selection))
            .expect("rust codegen");
    let rust = output;

    assert!(rust
        .code
        .contains("#[ignore = \"faber: omitte - blocked by service\"]"));
    assert!(!rust
        .code
        .contains("not selected by name selected ignored case"));
}

#[test]
fn emits_main_body_and_scribe_as_println() {
    let mut interner = Interner::new();
    let salve = interner.intern("Salve, munde!");
    let types = TypeTable::new();

    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: HirId(1),
                kind: HirStmtKind::Expr(HirExpr {
                    id: HirId(2),
                    kind: HirExprKind::Scribe(
                        HirScribeKind::Nota,
                        vec![HirExpr {
                            id: HirId(3),
                            kind: HirExprKind::Literal(HirLiteral::String(salve)),
                            ty: None,
                            span: span(),
                        }],
                    ),
                    ty: None,
                    span: span(),
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("fn main() {"));
    assert!(rust.code.contains("println!(\"{}\", \"Salve, munde!\");"));
}

#[test]
fn emits_usage_driven_and_importa_use_statements() {
    let mut interner = Interner::new();
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let numerus = types.primitive(Primitive::Numerus);
    let map_ty = types.map(textus, numerus);

    let path = interner.intern("std/collections");
    let name = interner.intern("HashMap");
    let alias_name = interner.intern("Mapa");

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(10),
                def_id: DefId(10),
                kind: HirItemKind::Import(HirImport {
                    path,
                    visibility: Visibility::Private,
                    items: vec![HirImportItem { def_id: DefId(11), name, alias: Some(alias_name) }],
                }),
                span: span(),
            },
            HirItem {
                id: HirId(12),
                def_id: DefId(12),
                kind: HirItemKind::TypeAlias(HirTypeAlias { name: interner.intern("Tab"), ty: map_ty }),
                span: span(),
            },
        ],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("use std::collections::HashMap as Mapa;"));
    assert!(rust.code.contains("use std::collections::HashMap;"));
    assert!(!rust.code.contains("use std::collections::HashSet;"));
}

#[test]
fn traverses_match_patterns_and_closure_params_in_name_collection() {
    let mut interner = Interner::new();
    let enum_name = interner.intern("Res");
    let variant_name = interner.intern("Ok");
    let local_name = interner.intern("value");
    let bind_name = interner.intern("bound");
    let closure_name = interner.intern("p");
    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let match_expr = HirExpr {
        id: HirId(20),
        kind: HirExprKind::Discerne(
            vec![HirExpr { id: HirId(21), kind: HirExprKind::Path(DefId(40)), ty: Some(numerus), span: span() }],
            vec![HirCasuArm {
                patterns: vec![HirPattern::Variant(
                    DefId(30),
                    vec![HirPattern::Binding(DefId(41), bind_name)],
                )],
                guard: Some(HirExpr {
                    id: HirId(22),
                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                    ty: Some(types.primitive(Primitive::Bivalens)),
                    span: span(),
                }),
                body: HirExpr {
                    id: HirId(23),
                    kind: HirExprKind::Literal(HirLiteral::Int(7)),
                    ty: Some(numerus),
                    span: span(),
                },
                span: span(),
            }],
        ),
        ty: Some(numerus),
        span: span(),
    };

    let closure_expr = HirExpr {
        id: HirId(24),
        kind: HirExprKind::Clausura(
            vec![HirParam {
                def_id: DefId(42),
                name: closure_name,
                ty: numerus,
                mode: HirParamMode::Owned,
                optional: false,
                sponte: false,
                fixus: false,
                default: None,
                span: span(),
            }],
            None,
            None,
            Box::new(HirExpr { id: HirId(25), kind: HirExprKind::Path(DefId(42)), ty: Some(numerus), span: span() }),
        ),
        ty: None,
        span: span(),
    };

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(10),
                def_id: DefId(29),
                kind: HirItemKind::Enum(HirEnum {
                    name: enum_name,
                    type_params: Vec::new(),
                    variants: vec![HirVariant {
                        def_id: DefId(30),
                        name: variant_name,
                        fields: Vec::new(),
                        span: span(),
                    }],
                }),
                span: span(),
            },
            HirItem {
                id: HirId(15),
                def_id: DefId(31),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: interner.intern("collector"),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: None,
                    err_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![
                            HirStmt {
                                id: HirId(11),
                                kind: HirStmtKind::Local(crate::hir::HirLocal {
                                    def_id: DefId(40),
                                    name: local_name,
                                    ty: Some(numerus),
                                    init: Some(HirExpr {
                                        id: HirId(12),
                                        kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                        ty: Some(numerus),
                                        span: span(),
                                    }),
                                    mutable: false,
                                }),
                                span: span(),
                            },
                            HirStmt { id: HirId(13), kind: HirStmtKind::Expr(match_expr), span: span() },
                            HirStmt { id: HirId(14), kind: HirStmtKind::Expr(closure_expr), span: span() },
                        ],
                        expr: None,
                        span: span(),
                    }),
                    is_async: false,
                    is_generator: false,
                    test: None,
                }),
                span: span(),
            },
        ],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("match "));
    assert!(rust.code.contains("Ok"));
    assert!(rust.code.contains("|p|"));
}

#[test]
fn resolves_type_names_for_named_defs() {
    let mut interner = Interner::new();
    let iface_name = interner.intern("Servitium");
    let struct_name = interner.intern("Structura");
    let enum_name = interner.intern("Enumeratio");
    let alias_name = interner.intern("Alias");
    let mut types = TypeTable::new();
    let iface_ty = types.intern(Type::Interface(DefId(70)));
    let struct_ty = types.intern(Type::Struct(DefId(71)));
    let enum_ty = types.intern(Type::Enum(DefId(72)));

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(58),
                def_id: DefId(71),
                kind: HirItemKind::Struct(HirStruct {
                    name: struct_name,
                    type_params: Vec::new(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    extends: None,
                    implements: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(59),
                def_id: DefId(72),
                kind: HirItemKind::Enum(HirEnum { name: enum_name, type_params: Vec::new(), variants: Vec::new() }),
                span: span(),
            },
            HirItem {
                id: HirId(60),
                def_id: DefId(70),
                kind: HirItemKind::Interface(HirInterface {
                    name: iface_name,
                    type_params: Vec::new(),
                    methods: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(61),
                def_id: DefId(73),
                kind: HirItemKind::TypeAlias(HirTypeAlias { name: alias_name, ty: struct_ty }),
                span: span(),
            },
            HirItem {
                id: HirId(62),
                def_id: DefId(74),
                kind: HirItemKind::Const(crate::hir::HirConst {
                    name: interner.intern("C"),
                    ty: Some(enum_ty),
                    value: HirExpr {
                        id: HirId(63),
                        kind: HirExprKind::Literal(HirLiteral::Int(0)),
                        ty: Some(types.primitive(Primitive::Numerus)),
                        span: span(),
                    },
                }),
                span: span(),
            },
            HirItem {
                id: HirId(67),
                def_id: DefId(75),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: interner.intern("iface_ret"),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(iface_ty),
                    err_ty: None,
                    body: None,
                    is_async: false,
                    is_generator: false,
                    test: None,
                }),
                span: span(),
            },
        ],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("Structura"));
    assert!(rust.code.contains("Enumeratio"));
    assert!(rust.code.contains("dyn Servitium"));
    assert!(rust.code.contains("pub const C: Enumeratio = 0;"));
}

#[test]
fn expr_codegen_handles_control_flow_and_operators() {
    let mut interner = Interner::new();
    let method = interner.intern("met");
    let field = interner.intern("fld");
    let numerus_name = interner.intern("N");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let bivalens = types.primitive(Primitive::Bivalens);
    let err_ty = types.intern(Type::Error);

    let support_program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(260),
                def_id: DefId(1),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: interner.intern("fn_name"),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: None,
                    err_ty: None,
                    body: None,
                    is_async: false,
                    is_generator: false,
                    test: None,
                }),
                span: span(),
            },
            HirItem {
                id: HirId(261),
                def_id: DefId(10),
                kind: HirItemKind::Struct(HirStruct {
                    name: interner.intern("Record"),
                    type_params: Vec::new(),
                    fields: vec![HirField {
                        def_id: DefId(251),
                        name: field,
                        ty: numerus,
                        is_static: false,
                        sponte: true,
                        fixus: false,
                        init: None,
                        span: span(),
                    }],
                    methods: Vec::new(),
                    extends: None,
                    implements: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(262),
                def_id: DefId(250),
                kind: HirItemKind::Enum(HirEnum {
                    name: interner.intern("Res"),
                    type_params: Vec::new(),
                    variants: vec![HirVariant {
                        def_id: DefId(5),
                        name: interner.intern("Case"),
                        fields: vec![HirVariantField { name: method, ty: numerus, span: span() }],
                        span: span(),
                    }],
                }),
                span: span(),
            },
        ],
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(263),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(2),
                        name: interner.intern("recv"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(264),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(3),
                        name: interner.intern("obj"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(265),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(4),
                        name: interner.intern("scrut"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(266),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(7),
                        name: interner.intern("iter_item"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(267),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(8),
                        name: interner.intern("lhs"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(268),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(9),
                        name: interner.intern("acc"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(269),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(12),
                        name: interner.intern("fut"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(270),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(13),
                        name: interner.intern("shared"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(271),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id: DefId(14),
                        name: interner.intern("ptr"),
                        ty: None,
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let iter_binding_name = interner.intern("n");
    let codegen = super::RustCodegen::new(&support_program, &interner);
    let mut w = codegen::CodeWriter::new();

    let expr = HirExpr {
        id: HirId(300),
        kind: HirExprKind::Tuple(vec![
            HirExpr {
                id: HirId(301),
                kind: HirExprKind::Binary(
                    crate::hir::HirBinOp::Add,
                    Box::new(HirExpr {
                        id: HirId(302),
                        kind: HirExprKind::Unary(
                            crate::hir::HirUnOp::Neg,
                            Box::new(HirExpr {
                                id: HirId(303),
                                kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(304),
                        kind: HirExprKind::Literal(HirLiteral::Int(2)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(305),
                kind: HirExprKind::Call(
                    Box::new(HirExpr { id: HirId(306), kind: HirExprKind::Path(DefId(1)), ty: None, span: span() }),
                    vec![HirCallArg {
                        name: None,
                        spread: false,
                        expr: HirExpr {
                            id: HirId(307),
                            kind: HirExprKind::Literal(HirLiteral::String(numerus_name)),
                            ty: Some(types.primitive(Primitive::Textus)),
                            span: span(),
                        },
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(308),
                kind: HirExprKind::MethodCall(
                    Box::new(HirExpr { id: HirId(309), kind: HirExprKind::Path(DefId(2)), ty: None, span: span() }),
                    method,
                    vec![HirCallArg {
                        name: None,
                        spread: false,
                        expr: HirExpr {
                            id: HirId(310),
                            kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                            ty: Some(bivalens),
                            span: span(),
                        },
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(311),
                kind: HirExprKind::Index(
                    Box::new(HirExpr {
                        id: HirId(312),
                        kind: HirExprKind::Field(
                            Box::new(HirExpr {
                                id: HirId(313),
                                kind: HirExprKind::Path(DefId(3)),
                                ty: None,
                                span: span(),
                            }),
                            field,
                        ),
                        ty: None,
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(314),
                        kind: HirExprKind::Literal(HirLiteral::Int(0)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(315),
                kind: HirExprKind::Si {
                    cond: Box::new(HirExpr {
                        id: HirId(316),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    then_block: HirBlock {
                        stmts: vec![HirStmt {
                            id: HirId(317),
                            kind: HirStmtKind::Expr(HirExpr {
                                id: HirId(318),
                                kind: HirExprKind::Literal(HirLiteral::Int(9)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                            span: span(),
                        }],
                        expr: None,
                        span: span(),
                    },
                    then_catch: None,
                    else_block: Some(HirBlock {
                        stmts: Vec::new(),
                        expr: Some(Box::new(HirExpr {
                            id: HirId(319),
                            kind: HirExprKind::Literal(HirLiteral::Int(10)),
                            ty: Some(numerus),
                            span: span(),
                        })),
                        span: span(),
                    }),
                },
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(320),
                kind: HirExprKind::Discerne(
                    vec![HirExpr {
                        id: HirId(321),
                        kind: HirExprKind::Path(DefId(4)),
                        ty: Some(numerus),
                        span: span(),
                    }],
                    vec![HirCasuArm {
                        patterns: vec![HirPattern::Variant(
                            DefId(5),
                            vec![HirPattern::Binding(DefId(6), method)],
                        )],
                        guard: Some(HirExpr {
                            id: HirId(322),
                            kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                            ty: Some(bivalens),
                            span: span(),
                        }),
                        body: HirExpr {
                            id: HirId(323),
                            kind: HirExprKind::Literal(HirLiteral::Float(1.5)),
                            ty: Some(types.primitive(Primitive::Fractus)),
                            span: span(),
                        },
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(324),
                kind: HirExprKind::Loop(HirBlock { stmts: Vec::new(), expr: None, span: span() }),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(325),
                kind: HirExprKind::Dum(
                    Box::new(HirExpr {
                        id: HirId(326),
                        kind: HirExprKind::Literal(HirLiteral::Bool(false)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    HirBlock { stmts: Vec::new(), expr: None, span: span() },
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(327),
                kind: HirExprKind::Itera(
                    HirIteraMode::Ex,
                    DefId(7),
                    iter_binding_name,
                    Box::new(HirExpr {
                        id: HirId(328),
                        kind: HirExprKind::Array(vec![HirArrayElement::Expr(HirExpr {
                            id: HirId(329),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        })]),
                        ty: None,
                        span: span(),
                    }),
                    HirBlock { stmts: Vec::new(), expr: None, span: span() },
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(330),
                kind: HirExprKind::Assign(
                    Box::new(HirExpr {
                        id: HirId(331),
                        kind: HirExprKind::Path(DefId(8)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(332),
                        kind: HirExprKind::Literal(HirLiteral::Int(11)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(333),
                kind: HirExprKind::AssignOp(
                    crate::hir::HirBinOp::Sub,
                    Box::new(HirExpr {
                        id: HirId(334),
                        kind: HirExprKind::Path(DefId(9)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(335),
                        kind: HirExprKind::Literal(HirLiteral::Int(4)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(336),
                kind: HirExprKind::Struct(
                    DefId(10),
                    vec![(
                        field,
                        HirExpr {
                            id: HirId(337),
                            kind: HirExprKind::Literal(HirLiteral::Int(7)),
                            ty: Some(numerus),
                            span: span(),
                        },
                    )],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(338),
                kind: HirExprKind::Clausura(
                    vec![HirParam {
                        def_id: DefId(11),
                        name: method,
                        ty: numerus,
                        mode: HirParamMode::Owned,
                        optional: false,
                        sponte: false,
                        fixus: false,
                        default: None,
                        span: span(),
                    }],
                    None,
                    None,
                    Box::new(HirExpr {
                        id: HirId(339),
                        kind: HirExprKind::Path(DefId(11)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(851),
                kind: HirExprKind::Scribe(
                    HirScribeKind::Nota,
                    vec![
                        HirExpr {
                            id: HirId(852),
                            kind: HirExprKind::Literal(HirLiteral::String(numerus_name)),
                            ty: Some(types.primitive(Primitive::Textus)),
                            span: span(),
                        },
                        HirExpr {
                            id: HirId(853),
                            kind: HirExprKind::Literal(HirLiteral::Int(3)),
                            ty: Some(numerus),
                            span: span(),
                        },
                    ],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(854),
                kind: HirExprKind::Adfirma(
                    Box::new(HirExpr {
                        id: HirId(855),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    Some(Box::new(HirExpr {
                        id: HirId(856),
                        kind: HirExprKind::Literal(HirLiteral::String(numerus_name)),
                        ty: Some(types.primitive(Primitive::Textus)),
                        span: span(),
                    })),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(857),
                kind: HirExprKind::Panic(Box::new(HirExpr {
                    id: HirId(858),
                    kind: HirExprKind::Literal(HirLiteral::String(numerus_name)),
                    ty: Some(types.primitive(Primitive::Textus)),
                    span: span(),
                })),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(340),
                kind: HirExprKind::Cede(Box::new(HirExpr {
                    id: HirId(341),
                    kind: HirExprKind::Path(DefId(12)),
                    ty: None,
                    span: span(),
                })),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(342),
                kind: HirExprKind::Verte {
                    source: Box::new(HirExpr {
                        id: HirId(343),
                        kind: HirExprKind::Literal(HirLiteral::Int(5)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    target: numerus,
                    entries: None,
                },
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(344),
                kind: HirExprKind::Ref(
                    crate::hir::HirRefKind::Mutable,
                    Box::new(HirExpr {
                        id: HirId(345),
                        kind: HirExprKind::Path(DefId(13)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(346),
                kind: HirExprKind::Deref(Box::new(HirExpr {
                    id: HirId(347),
                    kind: HirExprKind::Path(DefId(14)),
                    ty: None,
                    span: span(),
                })),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(348),
                kind: HirExprKind::Block(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(349),
                        kind: HirStmtKind::Local(crate::hir::HirLocal {
                            def_id: DefId(15),
                            name: field,
                            ty: Some(err_ty),
                            init: None,
                            mutable: true,
                        }),
                        span: span(),
                    }],
                    expr: Some(Box::new(HirExpr {
                        id: HirId(350),
                        kind: HirExprKind::Literal(HirLiteral::Nil),
                        ty: Some(err_ty),
                        span: span(),
                    })),
                    span: span(),
                }),
                ty: None,
                span: span(),
            },
        ]),
        ty: None,
        span: span(),
    };

    super::expr::generate_expr(&codegen, &expr, &types, &mut w, false, false, false).expect("expr codegen");
    let code = w.finish();

    assert!(code.contains("fn_name"));
    assert!(code.contains("met("));
    assert!(code.contains("match "));
    assert!(code.contains("loop "));
    assert!(code.contains("while "));
    assert!(code.contains("for iter_item in "));
    assert!(code.contains("Record"));
    assert!(code.contains("|met|"));
    assert!(code.contains(".await"));
    assert!(code.contains(" as i64"));
    assert!(code.contains("&mut "));
    assert!(code.contains("*ptr"));
    assert!(code.contains("\"N\""));
    assert!(code.contains("println!(\"{} {}\", \"N\".to_string(), 3)"));
    assert!(code.contains("assert!(true, \"{}\", \"N\".to_string())"));
    assert!(code.contains("panic!(\"{}\", \"N\".to_string())"));
}

#[test]
fn codegen_rejects_hir_error_nodes_for_all_targets() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: HirId(1),
                kind: HirStmtKind::Expr(HirExpr { id: HirId(2), kind: HirExprKind::Error, ty: None, span: span() }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let rust_error = match codegen::generate(Target::Rust, &program, &types, &interner) {
        Ok(_) => panic!("expected rust codegen error"),
        Err(error) => error,
    };
    assert!(rust_error
        .message
        .contains("HIR containing error expressions"));

    let faber_error = match codegen::generate(Target::Faber, &program, &types, &interner) {
        Ok(_) => panic!("expected faber codegen error"),
        Err(error) => error,
    };
    assert!(faber_error
        .message
        .contains("HIR containing error expressions"));
}

#[test]
fn direct_rust_codegen_propagates_entry_stmt_errors() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: HirId(1),
                kind: HirStmtKind::Expr(HirExpr { id: HirId(2), kind: HirExprKind::Error, ty: None, span: span() }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let rust_codegen = super::RustCodegen::new(&program, &interner);
    let error = match crate::codegen::Codegen::generate(&rust_codegen, &program, &types, &interner) {
        Ok(_) => panic!("expected direct rust codegen error"),
        Err(error) => error,
    };
    assert!(error
        .message
        .contains("cannot generate Rust for HIR error expression"));
}

#[test]
fn expr_codegen_block_propagates_nested_stmt_errors() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let support_program = HirProgram { items: Vec::new(), entry: None };
    let rust_codegen = super::RustCodegen::new(&support_program, &interner);
    let mut writer = super::CodeWriter::new();

    let block_expr = HirExpr {
        id: HirId(1),
        kind: HirExprKind::Block(HirBlock {
            stmts: vec![HirStmt {
                id: HirId(2),
                kind: HirStmtKind::Expr(HirExpr { id: HirId(3), kind: HirExprKind::Error, ty: None, span: span() }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
        ty: None,
        span: span(),
    };

    let error = match super::expr::generate_expr(&rust_codegen, &block_expr, &types, &mut writer, false, false, false) {
        Ok(_) => panic!("expected nested block codegen error"),
        Err(error) => error,
    };
    assert!(error
        .message
        .contains("cannot generate Rust for HIR error expression"));
}
