#[test]
fn rust_dynamic_object_values_use_faber_value() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ point ← { x = 10, label = "home" }
    nota point
    nota point.x
}
"#;

    let result = compiler.compile_str("dynamic-object.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("enum FaberValue"));
    assert!(rust.code.contains("HashMap<String, FaberValue>"));
    assert!(rust
        .code
        .contains(r#".insert("x".to_string(), FaberValue::from(10))"#));
    assert!(rust
        .code
        .contains(r#"point.get("x").cloned().unwrap_or_default()"#));
    assert!(!rust.code.contains("Box<dyn std::any::Any>"));
    assert!(!rust.code.contains("point.x"));
}

#[test]
fn rust_dynamic_context_coercions_use_faber_value() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio check(ignotum x) → textus {
    si x est nihil { redde "nil" }
    si verum x { redde "yes" }
    redde "other"
}

incipit {
    fixum _ data ← { count = 100, active = verum }
    ex data varia count, active
    count ← 200
    active ← falsum
    nota check(nihil)
    nota check(verum)
    nota check(42)
}
"#;

    let result = compiler.compile_str("dynamic-context.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("count = FaberValue::from(200)"));
    assert!(rust.code.contains("active = FaberValue::from(false)"));
    assert!(rust.code.contains("x == FaberValue::Nihil"));
    assert!(rust.code.contains("x == FaberValue::from(true)"));
    assert!(rust.code.contains("check(FaberValue::Nihil)"));
    assert!(rust.code.contains("check(FaberValue::from(true))"));
    assert!(rust.code.contains("check(FaberValue::from(42))"));
}

#[test]
fn rust_dynamic_collection_methods_coerce_arguments_to_faber_value() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    varia lista<ignotum> values ← vacua
    values.appende(1)
    varia tabula<textus, ignotum> data ← vacua
    data.pone("count", 2)
}
"#;

    let result = compiler.compile_str("dynamic-collection-methods.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("values.push(FaberValue::from(1));"));
    assert!(rust
        .code
        .contains(r#"data.insert("count".to_string(), FaberValue::from(2));"#));
}

#[test]
fn rust_object_spread_and_empty_object_use_dynamic_values() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ empty ← {}
    fixum _ base ← { a = 1, b = 2 }
    fixum _ extended ← { sparge base, c = 3 }
    nota empty
    nota extended
}
"#;

    let result = compiler.compile_str("dynamic-object-spread.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("let empty: HashMap<String, FaberValue>"));
    assert!(rust
        .code
        .contains("let extended: HashMap<String, FaberValue>"));
    assert!(rust.code.contains("__faber_verte_map_"));
    assert!(rust.code.contains(".extend(base)"));
    assert!(rust
        .code
        .contains(r#".insert("c".to_string(), FaberValue::from(3))"#));
    assert!(!rust.code.contains("HashMap<String, _>"));
}
