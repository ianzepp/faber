#[test]
fn direct_spread_call_expands_array_arguments() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio add(numerus a, numerus b) → numerus {
    redde a + b
}

incipit {
    fixum numerus[] numbers ← [3, 7]
    fixum _ sum ← add(sparge numbers)
    nota sum
}
"#;

    let result = compiler.compile_str("direct-spread-call.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("let sum: i64 = add((numbers[0usize].clone()), (numbers[1usize].clone()));"));
    assert!(!rust.code.contains("add(numbers.clone())"));
}
#[test]
fn clones_owned_path_arguments_for_function_calls() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio total(lista<numerus> nums) → numerus {
    redde 0
}

incipit {
    fixum _ numbers ← [1, 2, 3]
    nota total(numbers)
    nota total(numbers)
    nota total([4, 5, 6])
}
"#;

    let result = compiler.compile_str("call-arg-clone.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("total(numbers.clone())"));
    assert!(rust.code.contains("total(vec![4, 5, 6])"));
    assert!(!rust.code.contains("total(vec![4, 5, 6].clone())"));
}
