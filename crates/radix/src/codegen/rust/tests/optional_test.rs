#[test]
fn emits_optional_chain_for_plain_and_optional_receivers() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Address {
    textus city
    textus state sponte
}

genus User {
    textus name
    Address address sponte
}

incipit {
    fixum _ alice ← User {
        name = "Alice",
        address = Address { city = "Roma", state = "Italia" }
    }
    fixum _ city ← alice?.address?.city
    fixum _ state ← alice?.address?.state
    fixum _ bob ← User { name = "Bob" }
    fixum _ bobCity ← bob?.address?.city
    fixum _ items ← ["a", "b", "c"]
    nota city, state, bobCity, items?[10]
}
"#;

    let result = compiler.compile_str("optional-chain-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("let city: Option<String> = (alice.address.clone()).as_ref().map"));
    assert!(rust
        .code
        .contains("let state: Option<String> = (alice.address.clone()).as_ref().and_then"));
    assert!(rust
        .code
        .contains("let bobCity: Option<String> = (bob.address.clone()).as_ref().map"));
    assert!(rust.code.contains("(items).get((10) as usize).cloned()"));
    assert!(!rust.code.contains("(alice).as_ref()"));
    assert!(!rust.code.contains("(items).as_ref()"));
}

#[test]
fn rejects_ordinary_access_on_optional_receivers() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Address {
    textus city
}

genus User {
    Address address sponte
}

incipit {
    fixum _ user ← User {}
    fixum _ city ← user.address.city
}
"#;

    let result = compiler.compile_str("ordinary-optional-access.fab", source);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("optional receiver requires optional chaining")));
}

#[test]
fn emits_optional_parameters_with_defaults_at_direct_call_sites() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio greet(textus nomen, textus titulus sponte) → textus {
    si titulus est nihil {
        redde nomen
    }
    redde "§ §"(titulus, nomen)
}

functio paginate(numerus pagina sponte vel 1, numerus per_pagina sponte vel 10) → numerus {
    redde pagina + per_pagina
}

incipit {
    nota greet("Marcus")
    nota greet("Marcus", "Dominus")
    nota paginate()
    nota paginate(5)
    nota paginate(5, 25)
}
"#;

    let result = compiler.compile_str("optional-params-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("fn greet(nomen: String, titulus: Option<String>) -> String"));
    assert!(rust
        .code
        .contains("fn paginate(pagina: i64, per_pagina: i64) -> i64"));
    assert!(rust.code.contains(r#"greet("Marcus".to_string(), None)"#));
    assert!(rust
        .code
        .contains(r#"greet("Marcus".to_string(), Some("Dominus".to_string()))"#));
    assert!(rust.code.contains("paginate(1, 10)"));
    assert!(rust.code.contains("paginate(5, 10)"));
    assert!(rust.code.contains("paginate(5, 25)"));
    assert!(rust.code.contains("(titulus).clone().unwrap()"));
}

#[test]
fn preserves_option_values_for_optional_targets() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio greet(textus nomen sponte) → vacuum {
    nota nomen
}

incipit {
    varia textus ∪ nihil maybe ← "Marcus"
    fixum textus ∪ nihil alias ← maybe
    greet(maybe)
    greet("Julia")
}
"#;

    let result = compiler.compile_str("optional-target-values-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains(r#"let mut maybe: Option<String> = Some("Marcus".to_string());"#));
    assert!(rust.code.contains("let alias: Option<String> = maybe;"));
    assert!(rust.code.contains("greet(maybe);"));
    assert!(rust.code.contains(r#"greet(Some("Julia".to_string()));"#));
    assert!(!rust.code.contains("Some(maybe)"));
}

#[test]
fn emits_option_shaped_if_expression_branches() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    varia _ maybe ← nihil ∷ textus ∪ nihil
    fixum _ result ← nonnihil maybe sic maybe secus "default"
    nota result
}
"#;

    let result = compiler.compile_str("optional-if-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("let result: Option<String> = if maybe != None"));
    assert!(rust.code.contains("maybe.clone()"));
    assert!(rust.code.contains(r#"Some("default".to_string())"#));
    assert!(!rust.code.contains("Some(if maybe != None"));
}
#[test]
fn rust_optional_member_access_uses_map_lookup_and_nil_none() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ maybe ← { present = { value = 100 } }
    nota maybe?.present?.value
    fixum _ empty ← nihil
    fixum _ missing ← empty?.missing
    nota missing
}
"#;

    let result = compiler.compile_str("optional-member-access.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains(r#"(maybe).get("present").cloned()"#));
    assert!(rust
        .code
        .contains(r#").as_ref().and_then(|__faber_opt| __faber_opt.get("value").cloned())"#));
    assert!(rust.code.contains("let empty: Option<()> = None;"));
    assert!(rust
        .code
        .contains("let missing: Option<FaberValue> = None::<FaberValue>;"));
    assert!(!rust.code.contains(".present"));
    assert!(!rust.code.contains(".missing"));
    assert!(!rust.code.contains("Option</* error */>"));
}
#[test]
fn emits_non_consuming_option_coalesce() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum textus ∪ nihil name ← nihil
    fixum _ first ← name vel "Anonymous"
    fixum _ second ← name vel "Default"
    nota first
    nota second
}
"#;

    let result = compiler.compile_str("option-coalesce.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("(name).clone().unwrap_or(\"Anonymous\".to_string())"));
    assert!(rust
        .code
        .contains("(name).clone().unwrap_or(\"Default\".to_string())"));
    assert!(!rust.code.contains("(name).unwrap_or("));
}

#[test]
fn wraps_nullable_return_values_in_some() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ergo redde nihil
    redde a / b
}

functio first(lista<numerus> items, numerus target) → numerus ∪ nihil {
    itera ex items fixum item {
        si item ≡ target ergo redde item
    }
    redde nihil
}

functio keep(textus ∪ nihil value) → textus ∪ nihil {
    redde value
}

incipit {
    varia _ maybe ← nihil ∷ textus ∪ nihil
    nota divide(10, 2), first([1, 2, 3], 2), keep(maybe)
}
"#;

    let result = compiler.compile_str("nullable-return.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("return Some(a / b);"));
    assert!(rust.code.contains("return Some(item);"));
    assert!(rust.code.contains("return value;"));
    assert!(rust.code.contains("let mut maybe: Option<String> = None;"));
    assert!(!rust.code.contains("Some(None)"));
}
