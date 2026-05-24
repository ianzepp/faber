#[test]
fn ad_helper_preserves_native_unresolved_provider_behavior() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "host:echo" ("salve") → textus echoed ⇥ textus {
        nota echoed
    } cape err {
        nota err
    }
}
"#;

    let result = compiler.compile_str("ad-native-helper.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("#[cfg(not(target_arch = \"wasm32\"))]"));
    assert!(rust
        .code
        .contains("fn __faber_ad<T, A>(capability: &str, _args: A) -> Result<T, String>"));
    assert!(rust
        .code
        .contains("Err(format!(\"E_NO_ROUTE: unresolved capability {}\", capability))"));
    assert!(rust
        .code
        .contains("__faber_ad::<String, _>(\"host:echo\", (\"salve\".to_string(),))"));
}

#[test]
fn ad_helper_emits_wasm_host_syscall_import_and_route_codes() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    ad "host:echo" ("salve") → textus echoed ⇥ textus {
        nota echoed
    } cape err {
        nota err
    }

    ad "pg:query" ("select 1") → textus row ⇥ textus {
        nota row
    } cape err {
        nota err
    }
}
"#;

    let result = compiler.compile_str("ad-wasm-helper.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("#[cfg(target_arch = \"wasm32\")]"));
    assert!(rust.code.contains("#[link(wasm_import_module = \"\")]"));
    assert!(rust.code.contains("#[link_name = \"capability-call\"]"));
    assert!(rust
        .code
        .contains("fn __faber_syscall(route_code: i32) -> i32;"));
    assert!(rust
        .code
        .contains("let status = unsafe { __faber_syscall(route_code) };"));
    assert!(rust.code.contains("\"host:echo\" => Ok(1),"));
    assert!(rust.code.contains("\"pg:query\" => Ok(2),"));
    assert!(rust.code.contains("T: Default,"));
    assert!(rust
        .code
        .contains("__faber_ad::<String, _>(\"pg:query\", (\"select 1\".to_string(),))"));
}
