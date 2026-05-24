#[test]
fn emits_array_spread_without_moving_source_vector() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ first ← [1, 2, 3]
    fixum _ combined ← [sparge first]
    fixum _ extended ← [0, sparge first, 99]
    nota combined
    nota extended
}
"#;

    let result = compiler.compile_str("array-spread.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains(".extend((first).iter().cloned());"));
    assert!(!rust.code.contains(".extend(first);"));
}

#[test]
fn emits_typed_map_literals_assignments_and_primus() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    varia tabula<textus, numerus> cache ← vacua
    cache["foo"] ← 42
    fixum tabula<textus, numerus> scores ← { alice = 95, bob = 87 }
    fixum _ nums ← [1, 2, 3] ∷ lista<numerus>
    nota cache["foo"], scores["alice"], nums.primus()
}
"#;

    let result = compiler.compile_str("innatum-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("std::collections::HashMap::<String, i64>::new()"));
    assert!(rust.code.contains(r#"cache.insert("foo".to_string(), 42)"#));
    assert!(rust.code.contains(r#".insert("alice".to_string(), 95)"#));
    assert!(rust.code.contains("nums.first().cloned()"));
    assert!(!rust.code.contains("Box<dyn std::any::Any>"));
    assert!(!rust.code.contains(".primus()"));
}
#[test]
fn emits_itera_de_keys_and_indices_for_rust() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio hasKey(tabula<textus, numerus> obj, textus key) → bivalens {
    itera de obj fixum k {
        si k ≡ key ergo redde verum
    }
    redde falsum
}

incipit {
    fixum _ numbers ← [10, 20, 30]
    itera de numbers fixum index {
        nota numbers[index]
    }
}
"#;

    let result = compiler.compile_str("itera-de-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("for k in (obj).keys().cloned()"));
    assert!(rust
        .code
        .contains("for index in 0..((numbers).len() as i64)"));
    assert!(rust.code.contains("numbers[(index) as usize]"));
    assert!(!rust.code.contains("for k in obj"));
    assert!(!rust.code.contains("for index in numbers"));
}
#[test]
fn emits_textus_match_scrutinee_and_longitudo() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio code(textus name) → numerus {
    elige name {
        casu "textus" ergo redde 1
        ceterum ergo redde 0
    }
}

functio length(textus value) → numerus {
    redde value.longitudo()
}
"#;

    let result = compiler.compile_str("textus-match.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("match name.as_str()"));
    assert!(rust.code.contains("\"textus\" =>"));
    assert!(rust.code.contains("return value.len() as i64;"));
    assert!(!rust.code.contains("value.longitudo()"));
}
#[test]
fn emits_lista_morphology_methods_as_rust_collection_operations() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    varia _ items ← [1, 2, 3, 4, 5] ∷ numerus[]
    fixum _ evens ← items.filtrata(numerus x ∴ x % 2 ≡ 0)
    fixum _ extended ← items.addita(6)
    fixum _ reversed ← items.inversa()
    varia _ mutable ← [1, 2, 3]
    mutable.inverte()
    fixum _ sorted ← items.ordinata()
    fixum _ doubled ← items.mappata(numerus x ∴ x * 2)
    nota evens, extended, reversed, mutable, sorted, doubled
}
"#;

    let result = compiler.compile_str("lista-morphology.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("let evens: Vec<i64> = { let mut __faber_pred_"));
    assert!(rust.code.contains(".filter(|__faber_item| __faber_pred_"));
    assert!(rust
        .code
        .contains("let extended: Vec<i64> = { let mut __faber_list_"));
    assert!(rust.code.contains(".push(6);"));
    assert!(rust
        .code
        .contains("let reversed: Vec<i64> = { let mut __faber_list_"));
    assert!(rust.code.contains(".reverse();"));
    assert!(rust.code.contains("mutable.reverse();"));
    assert!(rust
        .code
        .contains("let sorted: Vec<i64> = { let mut __faber_list_"));
    assert!(rust.code.contains(".sort();"));
    assert!(rust
        .code
        .contains("let doubled: Vec<i64> = { let mut __faber_map_"));
    assert!(rust.code.contains(".map(|__faber_item| __faber_map_"));
    assert!(!rust.code.contains(".filtrata("));
    assert!(!rust.code.contains(".addita("));
    assert!(!rust.code.contains(".inversa("));
    assert!(!rust.code.contains(".inverte("));
    assert!(!rust.code.contains(".ordinata("));
    assert!(!rust.code.contains(".mappata("));
    assert!(!rust.code.contains("let doubled: Vec<_>"));
}
#[test]
fn clones_owned_array_values_from_indexed_locals() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ matrix ← [[1, 2], [3, 4]]
    fixum [row1, row2] ← matrix
    nota row1
    nota row2
}
"#;

    let result = compiler.compile_str("array-destructure-clone.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("let row1: Vec<i64> = matrix[(0) as usize].clone();"));
    assert!(rust
        .code
        .contains("let row2: Vec<i64> = matrix[(1) as usize].clone();"));
}
#[test]
fn emits_borrowed_iteration_for_lista_itera_ex() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
incipit {
    fixum _ numbers ← [1, 2, 3]
    itera ex numbers fixum n {
        nota n
    }
    nota numbers
}
"#;

    let result = compiler.compile_str("borrowed-itera-ex.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("for __faber_item_"));
    assert!(rust.code.contains(" in &(numbers)"));
    assert!(rust.code.contains(".clone();"));
    assert!(!rust.code.contains("for n in numbers"));
}
#[test]
fn emits_usize_cast_for_lista_indexing() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
functio pick(lista<numerus> items, numerus index) → numerus {
    redde items[index]
}
"#;

    let result = compiler.compile_str("lista-index.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("return items[(index) as usize];"));
    assert!(!rust.code.contains("return items[index];"));
}
